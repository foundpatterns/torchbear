
-- Patch require to log all executed modules
_G._require = require
function _G.require (module_name)
    if package.loaded[module_name] == nil then
        _log.trace("[running] " .. module_name)
    end
    return _require(module_name)
end

function _G.require_time(module_name)
    local start_time = os.clock()
    local module = _require(module_name)
    local elapsed = (os.clock() - start_time) * 1000
    log.info(module_name .. " elapsed " .. elapsed .. " milliseconds")
    return module
end


xpcall(function ()
    local init_f, err = loadfile(torchbear.init_filename)
    if not init_f then error(err) end

    local handler = init_f()

    if handler then
        torchbear.handler = handler
    end

end, function (msg)
    msg = tostring(msg)
    local trace = debug.traceback(msg, 3)
    _log.error(trace)
end)

if not torchbear.handler then
    _log.debug("No handler specified")
end
