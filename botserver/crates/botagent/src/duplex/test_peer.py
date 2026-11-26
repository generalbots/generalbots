import json, sys, threading, time
mode = sys.argv[1]
arg = sys.argv[2] if len(sys.argv) > 2 else ""
inp, out = sys.stdin.buffer, sys.stdout.buffer
def put(b):
    out.write(b + b"\n")
    out.flush()
def send(o):
    put(json.dumps(o, ensure_ascii=False).encode("utf-8"))
def beat(path):
    def run():
        while True:
            with open(path, "a") as f:
                f.write("x")
            time.sleep(0.05)
    threading.Thread(target=run, daemon=True).start()
if mode == "echo":
    for raw in inp:
        put(b"ECHO " + raw.rstrip(b"\r\n"))
    sys.stderr.write("peer: stdin closed\n")
elif mode == "exit3":
    sys.stderr.write("peer: failing on purpose\n")
    sys.stderr.flush()
    put(b"bye")
    sys.exit(3)
elif mode == "big":
    put(b"a" * int(arg))
    inp.read()
elif mode == "stubborn":
    beat(arg)
    put(b"ready")
    inp.read()
    while True:
        time.sleep(1)
elif mode == "tree":
    import subprocess
    code = "import sys, time\nwhile True:\n    open(sys.argv[1], 'a').write('x')\n    time.sleep(0.05)\n"
    subprocess.Popen([sys.executable, "-c", code, arg])
    put(b"ready")
    inp.read()
    while True:
        time.sleep(1)
elif mode == "rpc":
    for raw in inp:
        m = json.loads(raw)
        meth, rid, p = m.get("method"), m.get("id"), m.get("params")
        if meth is None or rid is None:
            continue
        if meth == "echo":
            send({"id": rid, "result": p})
        elif meth == "fail":
            send({"id": rid, "error": {"code": -32000, "message": "nope", "data": p}})
        elif meth == "notify3":
            for n in range(3):
                send({"method": "tick", "params": {"n": n}})
            send({"id": rid, "result": "done"})
        elif meth == "ask":
            send({"jsonrpc": "2.0", "id": "s-1", "method": p["method"], "params": {"q": "ação"}})
            send({"id": rid, "result": json.loads(inp.readline())})
        elif meth == "die":
            sys.exit(5)
