import json, sys
mode = sys.argv[1]
STATE = {}
def send(o):
    sys.stdout.write(json.dumps(o) + "\n")
    sys.stdout.flush()
def note(method, params):
    send({"method": method, "params": params})
def read():
    line = sys.stdin.buffer.readline()
    if not line:
        sys.exit(0)
    return json.loads(line)
def wait_reply(rid):
    while True:
        m = read()
        if m.get("id") == rid and "method" not in m:
            return m
        if "method" in m and "id" in m:
            answer(m)
def completed(status, error=None):
    note("turn/completed", {"threadId": "t", "turn": {"id": "turn-1", "status": status, "error": error}})
def message(text, delta=False):
    if delta:
        note("item/agentMessage/delta", {"itemId": "msg-1", "delta": text[:3]})
        note("item/agentMessage/delta", {"itemId": "msg-1", "delta": text[3:]})
    note("item/completed", {"item": {"type": "agentMessage", "id": "msg-1", "text": text}})
def ask(rid, method, params):
    send({"id": rid, "method": method, "params": params})
    if mode == "vanish":
        sys.exit(0)
    return wait_reply(rid).get("result", {}).get("decision")
CMD = {"type": "commandExecution", "id": "call_1", "command": "/bin/bash -lc 'echo olá'", "cwd": "/home/codex", "status": "inProgress"}
CHANGES = [{"path": "/home/codex/a.txt", "kind": "add", "diff": "+a"}, {"path": "/home/codex/b.txt", "kind": {"type": "delete"}, "diff": ""}]
def turn(params):
    text = params["input"][0]["text"]
    if mode in ("allow", "vanish"):
        note("item/started", {"item": CMD})
        decision = ask(100, "item/commandExecution/requestApproval", {"itemId": "call_1", "threadId": "t", "turnId": "turn-1", "command": CMD["command"], "cwd": "/home/codex", "startedAtMs": 1})
        if decision == "accept":
            done = dict(CMD, status="completed", exitCode=0, aggregatedOutput="olá\n")
        else:
            done = dict(CMD, status="declined")
        note("item/completed", {"item": done})
        message("decision=%s" % decision, delta=True)
        completed("completed" if decision != "cancel" else "interrupted")
    elif mode == "file":
        item = {"type": "fileChange", "id": "patch_1", "changes": CHANGES, "status": "inProgress"}
        note("item/started", {"item": item})
        decision = ask(101, "item/fileChange/requestApproval", {"itemId": "patch_1", "threadId": "t", "turnId": "turn-1", "startedAtMs": 1})
        note("item/completed", {"item": dict(item, status="completed" if decision == "accept" else "declined")})
        message("decision=%s" % decision)
        completed("completed")
    elif mode == "unknown":
        send({"id": 200, "method": "item/tool/requestUserInput", "params": {"itemId": "q1", "questions": []}})
        reply = wait_reply(200)
        message("reply=%s" % reply.get("error", {}).get("code"))
        completed("completed")
    elif mode == "limit":
        note("error", {"error": {"message": "Reconnecting... 1/5"}, "willRetry": True})
        completed("failed", {"message": "You've hit your usage limit. Try again at Nov 22nd.", "codexErrorInfo": "usageLimitExceeded"})
    elif mode == "unasked":
        note("item/started", {"item": CMD})
        note("item/completed", {"item": dict(CMD, status="completed", exitCode=0, aggregatedOutput="")})
    elif mode == "stray":
        message("got=%s stray=%s" % (text, STATE.get("stray")))
        completed("completed")
        send({"id": 300, "method": "item/commandExecution/requestApproval", "params": {"itemId": "call_9", "threadId": "t", "turnId": "turn-1", "command": "/bin/bash -lc 'rm -rf x'", "cwd": "/home/codex", "startedAtMs": 1}})
    else:
        message("got=%s" % text)
        completed("completed")
def answer(m):
    meth, rid = m["method"], m["id"]
    if meth == "initialize":
        send({"id": rid, "result": {"userAgent": "gb_bridge/0.171.0 (Test; x86_64) (gb_bridge; 1)"}})
    elif meth == "account/read":
        send({"id": rid, "result": {"account": None if mode == "noauth" else {"type": "chatgpt"}, "requiresOpenaiAuth": True}})
    elif meth == "thread/start":
        send({"id": rid, "result": {"thread": {"id": "01a2-new"}}})
    elif meth == "thread/resume":
        if m["params"].get("threadId") == "01a2-stored":
            send({"id": rid, "result": {"thread": {"id": "01a2-stored"}}})
        else:
            send({"id": rid, "error": {"code": -32600, "message": "no such thread"}})
    elif meth == "turn/start":
        send({"id": rid, "result": {"turn": {"id": "turn-1", "status": "inProgress"}}})
        turn(m["params"])
    elif meth == "turn/interrupt":
        send({"id": rid, "result": {}})
        completed("interrupted")
    else:
        send({"id": rid, "error": {"code": -32601, "message": "unknown"}})
if mode == "resume":
    sys.stdout.write("GB-THREAD 01a2-stored\n")
    sys.stdout.flush()
while True:
    m = read()
    if "method" in m and "id" in m:
        answer(m)
    elif m.get("id") == 300:
        STATE["stray"] = m.get("error", {}).get("code")
