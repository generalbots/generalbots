import json, sys
mode = sys.argv[1]
def send(o):
    sys.stdout.write(json.dumps(o) + "\n")
    sys.stdout.flush()
def read():
    line = sys.stdin.readline()
    return json.loads(line) if line else None
def assistant(block):
    send({"type": "assistant", "message": {"model": "claude-test", "role": "assistant", "content": [block]}, "parent_tool_use_id": None})
def ask(rid, command, tid):
    send({"type": "control_request", "request_id": rid, "request": {"subtype": "can_use_tool", "tool_name": "Bash", "input": {"command": command}, "tool_use_id": tid}})
init = read()
assert init["request"]["subtype"] == "initialize"
send({"type": "control_response", "response": {"subtype": "success", "request_id": init["request_id"], "response": {}}})
stray = None
n = 0
while True:
    m = read()
    if m is None:
        break
    if m.get("type") == "control_response" and m["response"]["request_id"] == "req-stray":
        stray = m["response"]["response"]["behavior"]
        continue
    if m.get("type") != "user":
        continue
    n += 1
    send({"type": "system", "subtype": "init", "session_id": "sess-1", "cwd": "/home/claude"})
    assistant({"type": "thinking", "thinking": "pondering %d" % n})
    tid = "toolu_%d" % n
    assistant({"type": "tool_use", "id": tid, "name": "Bash", "input": {"command": "echo %d" % n}})
    ask("req-%d" % n, "echo %d" % n, tid)
    behavior = read()["response"]["response"]["behavior"]
    if behavior == "allow":
        send({"type": "user", "message": {"role": "user", "content": [{"type": "tool_result", "tool_use_id": tid, "content": "ok", "is_error": False}]}, "parent_tool_use_id": None})
    assistant({"type": "text", "text": "turn %d %s stray=%s" % (n, behavior, stray)})
    send({"type": "result", "subtype": "success", "is_error": False, "result": "", "session_id": "sess-1", "permission_denials": []})
    if mode == "stray" and n == 1:
        ask("req-stray", "rm -rf /", "toolu_x")
