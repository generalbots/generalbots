import json, sys
mode = sys.argv[1]
inp, out = sys.stdin, sys.stdout
def send(o):
    out.write(json.dumps(o) + "\n")
    out.flush()
def read():
    line = inp.readline()
    return json.loads(line) if line else None
def text(t, model="claude-test"):
    send({"type": "assistant", "message": {"model": model, "role": "assistant", "content": [{"type": "text", "text": t}]}, "parent_tool_use_id": None})
def tool_use(tid, name, args):
    send({"type": "assistant", "message": {"model": "claude-test", "role": "assistant", "content": [{"type": "tool_use", "id": tid, "name": name, "input": args}]}, "parent_tool_use_id": None})
def tool_result(tid, content, err):
    send({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": tid, "content": content, "is_error": err}]}, "parent_tool_use_id": None})
def ask(rid, name, args, tid):
    send({"type": "control_request", "request_id": rid, "request": {"subtype": "can_use_tool", "tool_name": name, "display_name": name, "input": args, "tool_use_id": tid}})
def result(sub="success", err=False, res="", sid="sess-1"):
    send({"type": "result", "subtype": sub, "is_error": err, "result": res, "session_id": sid, "permission_denials": []})
init = read()
assert init["type"] == "control_request" and init["request"]["subtype"] == "initialize"
send({"type": "control_response", "response": {"subtype": "success", "request_id": init["request_id"], "response": {}}})
user = read()
assert user["type"] == "user"
prompt = user["message"]["content"]
send({"type": "system", "subtype": "init", "session_id": "sess-1", "cwd": "/home/claude", "permissionMode": "default"})
if mode == "flow":
    text("Working on: " + prompt[:40])
    tool_use("toolu_1", "Bash", {"command": "ls /nope"})
    ask("req-1", "Bash", {"command": "ls /nope"}, "toolu_1")
    r = read()["response"]
    assert r["request_id"] == "req-1"
    b = r["response"]
    if b["behavior"] == "allow":
        assert b["updatedInput"] == {"command": "ls /nope"}
        tool_result("toolu_1", "Exit code 2\nls: cannot access '/nope'\n<system-reminder>x</system-reminder>", True)
        text("Done.")
    else:
        tool_result("toolu_1", b["message"], True)
        text("Denied: " + b["message"])
    result()
elif mode == "unknown":
    send({"type": "control_request", "request_id": "h1", "request": {"subtype": "hook_callback", "callback_id": "x"}})
    first = read()["response"]
    ask("req-2", "CronCreate", {"cron": "* * * * *"}, "toolu_2")
    second = read()["response"]
    text("peer saw: %s/%s/%s" % (first["subtype"], first["request_id"], second["response"]["behavior"]))
    result()
elif mode == "midway":
    tool_use("toolu_3", "Write", {"file_path": "notes.txt", "content": "x"})
    ask("req-3", "Write", {"file_path": "notes.txt", "content": "x"}, "toolu_3")
    sys.exit(3)
elif mode == "unasked":
    tool_use("toolu_9", "Bash", {"command": "true"})
    tool_result("toolu_9", "", False)
    text("ran it")
    result()
elif mode == "interrupt":
    text("thinking")
    while True:
        m = read()
        if m is None:
            sys.exit(4)
        if m.get("type") == "control_request" and m["request"]["subtype"] == "interrupt":
            send({"type": "control_response", "response": {"subtype": "success", "request_id": m["request_id"], "response": {}}})
            result("error_during_execution", True)
            break
elif mode == "synthetic":
    text("Not logged in · Please run /login", "<synthetic>")
    result("success", True, "Not logged in · Please run /login")
for _ in inp:
    pass
