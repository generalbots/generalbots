"""A scripted ACP agent for the bridge tests: answers one session/prompt.

Modes: plain (thought and text chunks), ask (asks to run a command, then
reports it), unasked (runs a command without asking), edit (asks for one
file edit), stray (asks for permission after its answer, between prompts,
and tells the next prompt what it was answered). Prints the permission
outcome it received as text.
"""
import json
import sys
import time

MODE = sys.argv[1] if len(sys.argv) > 1 else "plain"
NEXT_ID = [100]
STRAY = [None]


def send(message):
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def update(session, body):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": session, "update": body}})


def chunk(session, kind, text):
    update(session, {"sessionUpdate": kind, "content": {"type": "text", "text": text}})


def ask(session, call):
    NEXT_ID[0] += 1
    request_id = NEXT_ID[0]
    send({"jsonrpc": "2.0", "id": request_id, "method": "session/request_permission", "params": {
        "sessionId": session,
        "toolCall": call,
        "options": [
            {"optionId": "yes-always", "name": "Always", "kind": "allow_always"},
            {"optionId": "yes", "name": "Allow", "kind": "allow_once"},
            {"optionId": "no", "name": "Reject", "kind": "reject_once"},
        ],
    }})
    for line in sys.stdin:
        message = json.loads(line)
        if message.get("id") == request_id:
            return message.get("result", {}).get("outcome", {})
    return {}


def prompt(request):
    session = request["params"]["sessionId"]
    blocks = request["params"]["prompt"]
    if MODE == "plain":
        chunk(session, "agent_thought_chunk", "pensando")
        kinds = ",".join(block["type"] for block in blocks)
        chunk(session, "agent_message_chunk", "Olá, ")
        chunk(session, "agent_message_chunk", "blocos " + kinds)
    elif MODE in ("ask", "edit"):
        if MODE == "ask":
            call = {"toolCallId": "t1", "title": "Run true", "kind": "execute", "status": "pending",
                    "rawInput": {"command": "true"}}
        else:
            call = {"toolCallId": "t1", "title": "Edit a.txt", "kind": "edit", "status": "pending",
                    "locations": [{"path": "/home/antigravity/a.txt"}]}
        update(session, dict(call, sessionUpdate="tool_call"))
        outcome = ask(session, call)
        chosen = outcome.get("optionId", outcome.get("outcome", ""))
        if chosen == "yes":
            update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "t1", "status": "completed",
                             "content": [{"type": "content", "content": {"type": "text", "text": "done"}}]})
        chunk(session, "agent_message_chunk", "escolha " + chosen)
    elif MODE == "unasked":
        update(session, {"sessionUpdate": "tool_call", "toolCallId": "t9", "title": "rm", "kind": "execute",
                         "status": "in_progress"})
        update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "t9", "status": "completed"})
        chunk(session, "agent_message_chunk", "feito")
    elif MODE == "search":
        update(session, {"sessionUpdate": "tool_call", "toolCallId": "s1", "title": "Search the web",
                         "kind": "search", "status": "in_progress", "rawInput": {"query": "rust 2026"}})
        update(session, {"sessionUpdate": "tool_call_update", "toolCallId": "s1", "status": "completed"})
        chunk(session, "agent_message_chunk", "achei")
    elif MODE == "stray":
        chunk(session, "agent_message_chunk", "stray=%s" % STRAY[0])
    send({"jsonrpc": "2.0", "id": request["id"], "result": {"stopReason": "end_turn"}})
    if MODE == "stray":
        # Only once the bridge is done with this prompt: between prompts.
        time.sleep(0.2)
        send({"jsonrpc": "2.0", "id": 900, "method": "session/request_permission", "params": {
            "sessionId": session,
            "toolCall": {"toolCallId": "t7", "title": "rm -rf /", "kind": "execute", "status": "pending",
                         "rawInput": {"command": "rm -rf /"}},
            "options": [{"optionId": "yes", "name": "Allow", "kind": "allow_once"}],
        }})


for line in sys.stdin:
    message = json.loads(line)
    method = message.get("method")
    if method == "session/prompt":
        prompt(message)
    elif method == "session/cancel":
        pass
    elif message.get("id") == 900 and not method:
        STRAY[0] = message.get("error", {}).get("code")
    elif "id" in message and method:
        send({"jsonrpc": "2.0", "id": message["id"], "result": {}})
