use rlua::{self, Lua, UserData, UserDataMethods};
use select::{self, node::Raw};
use std::{
    mem,
    sync::Arc
};
use rlua_serde;

/// Please read `select`'s documentation to know the meanings of each variants.
#[derive(Serialize, Deserialize)]
enum Predicate {
    Any,
    Text,
    Element,
    Comment,
    Class(String),
    Name(String),
    Attr(String, Option<String>),
    Not(Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    And(Box<Predicate>, Box<Predicate>),
    Child(Box<Predicate>, Box<Predicate>),
    Descendant(Box<Predicate>, Box<Predicate>),
}

impl<'a> select::predicate::Predicate for &'a Predicate {
    fn matches(&self, node: &select::node::Node<'_>) -> bool {
        <Predicate as select::predicate::Predicate>::matches(self, node)
    }
}

impl select::predicate::Predicate for Predicate {
    fn matches(&self, node: &select::node::Node<'_>) -> bool {
        use self::Predicate::*;
        match self {
            Any => select::predicate::Any.matches(node),
            Text => select::predicate::Text.matches(node),
            Element => select::predicate::Element.matches(node),
            Comment => select::predicate::Comment.matches(node),
            Class(s) => select::predicate::Class(s.as_str()).matches(node),
            Name(s) => select::predicate::Name(s.as_str()).matches(node),
            Attr(s, op) => match op {
                Some(ss) => select::predicate::Attr(s.as_str(), ss.as_str()).matches(node),
                None => select::predicate::Attr(s.as_str(), ()).matches(node),
            },
            Not(pred) => select::predicate::Not(pred.as_ref()).matches(node),
            And(a, b) => select::predicate::And(a.as_ref(), b.as_ref()).matches(node),
            Or(a, b) => select::predicate::Or(a.as_ref(), b.as_ref()).matches(node),
            Child(a, b) => select::predicate::Child(a.as_ref(), b.as_ref()).matches(node),
            Descendant(a, b) => select::predicate::Descendant(a.as_ref(), b.as_ref()).matches(node),
        }
    }
}

/// The way `select::node::Node` is defined makes it impossible to use it in LUA bindings.
/// Therefore here we need to redefine it.
/// The difference is:
/// - In `select::node::Node`, `document` is a reference, so the lifetime is statically managed by
/// Rust.
/// - Here, `document` is a reference counter - `std::sync::Arc`.
struct Node {
    document: Document,
    index: usize,
}

impl Node {
    fn to_node(&self) -> select::node::Node {
        select::node::Node::new(&self.document.0, self.index).unwrap()
    }
    fn with_index(&self, index: usize) -> Self {
        Node {
            document: self.document.clone(),
            index,
        }
    }
    fn parent(&self) -> Option<Self> {
        self.to_node().parent().map(|p| self.with_index(p.index()))
    }
    fn prev(&self) -> Option<Self> {
        self.to_node().prev().map(|p| self.with_index(p.index()))
    }
    fn next(&self) -> Option<Self> {
        self.to_node().next().map(|p| self.with_index(p.index()))
    }
    fn first_child(&self) -> Option<Self> {
        self.to_node()
            .first_child()
            .map(|p| self.with_index(p.index()))
    }
    fn last_child(&self) -> Option<Self> {
        self.to_node()
            .last_child()
            .map(|p| self.with_index(p.index()))
    }
}

fn to_owned(op: Option<&str>) -> Option<String> {
    op.map(|s| s.to_owned())
}

impl UserData for Node {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("index", |_, this, _: ()| Ok(this.to_node().index()));
        methods.add_method("name", |_, this, _: ()| Ok(to_owned(this.to_node().name())));
        methods.add_method("attr", |_, this, s: String| {
            Ok(to_owned(this.to_node().attr(&s)))
        });
        methods.add_method("parent", |_, this, _: ()| Ok(this.parent()));
        methods.add_method("prev", |_, this, _: ()| Ok(this.prev()));
        methods.add_method("next", |_, this, _: ()| Ok(this.next()));
        methods.add_method("first_child", |_, this, _: ()| Ok(this.first_child()));
        methods.add_method("last_child", |_, this, _: ()| Ok(this.last_child()));
        methods.add_method("text", |_, this, _: ()| Ok(this.to_node().text()));
        methods.add_method("html", |_, this, _: ()| Ok(this.to_node().html()));
        methods.add_method("inner_html", |_, this, _: ()| {
            Ok(this.to_node().inner_html())
        });
        methods.add_method("as_text", |_, this, _: ()| {
            Ok(to_owned(this.to_node().as_text()))
        });
        methods.add_method("as_comment", |_, this, _: ()| {
            Ok(to_owned(this.to_node().as_comment()))
        });

        methods.add_method("find", |_, this, val: rlua::Value| {
            let pred: Predicate = rlua_serde::from_value(val)?;
            let vec: Vec<_> = this
                .to_node()
                .find(pred)
                .map(|p| this.with_index(p.index()))
                .collect();
            Ok(vec)
        });
        methods.add_method("is", |_, this, val: rlua::Value| {
            let pred: Predicate = rlua_serde::from_value(val)?;
            Ok(this.to_node().is(pred))
        });

        methods.add_method("children", |_, this, _: ()| {
            let vec: Vec<_> = this
                .to_node()
                .children()
                .map(|p| this.with_index(p.index()))
                .collect();
            Ok(vec)
        });
        methods.add_method("descendants", |_, this, _: ()| {
            let vec: Vec<_> = this
                .to_node()
                .descendants()
                .map(|p| this.with_index(p.index()))
                .collect();
            Ok(vec)
        });
    }
}

#[derive(Clone)]
struct Document(Arc<select::document::Document>);

/// `select.rs` uses `Tendril`, a kind of shared buffer and is not `Send` by default.
/// `rlua` requires `Document` and `Node` to be `Send`, so here we need to call `into_send()`
/// to make the inner data `Send`.
/// `into_send()` returns `SendTendril`, which is a wrapper of `Tendril`,
/// so here we use transmute to convert the type.
fn into_send(raw: &mut Raw) {
    use select::node::Data;
    match raw.data {
        Data::Text(ref mut tendril) => {
            *tendril = unsafe { mem::transmute(tendril.clone().into_send()) };
        }
        Data::Comment(ref mut tendril) => {
            *tendril = unsafe { mem::transmute(tendril.clone().into_send()) };
        }
        Data::Element(_, ref mut vec) => {
            for (_, tendril) in vec {
                *tendril = unsafe { mem::transmute(tendril.clone().into_send()) };
            }
        }
    }
}

impl Document {
    fn from_str(text: &str) -> Document {
        let mut doc = select::document::Document::from(text);
        for raw in &mut doc.nodes {
            // This line is important
            into_send(raw);
        }
        Document(Arc::new(doc))
    }
}

/// Each elements in `Document` must be passed through `into_send()` to make this safe.
unsafe impl Send for Document {}

impl UserData for Document {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method("find", |_, this, val: rlua::Value| {
            let pred: Predicate = rlua_serde::from_value(val)?;
            let vec: Vec<_> = this
                .0
                .find(pred)
                .map(|node| Node {
                    document: this.clone(),
                    index: node.index(),
                }).collect();
            Ok(vec)
        });
    }
}

pub fn init(lua: &Lua) -> crate::Result<()> {
    let select = lua.create_table()?;

    // New Document from string
    select.set(
        "document",
        lua.create_function(|_, text: String| Ok(Document::from_str(text.as_str())))?,
    )?;

    // Create `Name` predicate
    select.set(
        "name",
        lua.create_function(|lua, text: String| {
            let pred = Predicate::Name(text);
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Class` predicate
    select.set(
        "class",
        lua.create_function(|lua, text: String| {
            let pred = Predicate::Class(text);
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Attr` predicate
    select.set(
        "attr",
        lua.create_function(|lua, args: (String, Option<String>)| {
            let pred = Predicate::Attr(args.0, args.1);
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Any` predicate
    select.set(
        "any",
        lua.create_function(|lua, _: ()| {
            let pred = Predicate::Any;
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Text` predicate
    select.set(
        "text",
        lua.create_function(|lua, _: ()| {
            let pred = Predicate::Text;
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Element` predicate
    select.set(
        "element",
        lua.create_function(|lua, _: ()| {
            let pred = Predicate::Element;
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Comment` predicate
    select.set(
        "comment",
        lua.create_function(|lua, _: ()| {
            let pred = Predicate::Comment;
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;

    // Create `Not` predicate
    select.set(
        "not",
        lua.create_function(|lua, pred: rlua::Value| {
            let pred: Predicate = rlua_serde::from_value(pred)?;
            let pred = Predicate::Not(Box::new(pred));
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `And` predicate
    select.set(
        "and",
        lua.create_function(|lua, (a, b): (rlua::Value, rlua::Value)| {
            let a: Predicate = rlua_serde::from_value(a)?;
            let b: Predicate = rlua_serde::from_value(b)?;
            let pred = Predicate::And(Box::new(a), Box::new(b));
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Or` predicate
    select.set(
        "or",
        lua.create_function(|lua, (a, b): (rlua::Value, rlua::Value)| {
            let a: Predicate = rlua_serde::from_value(a)?;
            let b: Predicate = rlua_serde::from_value(b)?;
            let pred = Predicate::Or(Box::new(a), Box::new(b));
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Child` predicate
    select.set(
        "child",
        lua.create_function(|lua, (a, b): (rlua::Value, rlua::Value)| {
            let a: Predicate = rlua_serde::from_value(a)?;
            let b: Predicate = rlua_serde::from_value(b)?;
            let pred = Predicate::Child(Box::new(a), Box::new(b));
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;
    // Create `Descendant` predicate
    select.set(
        "descendant",
        lua.create_function(|lua, (a, b): (rlua::Value, rlua::Value)| {
            let a: Predicate = rlua_serde::from_value(a)?;
            let b: Predicate = rlua_serde::from_value(b)?;
            let pred = Predicate::Descendant(Box::new(a), Box::new(b));
            rlua_serde::to_value(lua, &pred)
        })?,
    )?;

    let globals = lua.globals();
    globals.set("hquery", select)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rlua::{Lua, Value};

    #[test]
    fn test() {
        let lua = Lua::new();
        super::init(&lua).unwrap();
        lua.exec::<_, Value>(
            r#"
        local doc = hquery.document("<p>hello</p>")
        local vec = doc:find(hquery.name("p"))
        assert(vec[1]:text() == "hello")
        "#,
            None,
        ).unwrap();
    }
}
