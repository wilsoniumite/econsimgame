"""The core's tests, every test program at once.

usage: python crates/isostate-core/run_tests.py [--release] [--timeout SECONDS] [-- ARGS FOR EVERY TEST PROGRAM]

`cargo test -p isostate-core` builds some forty test programs (the library's own tests, the command's, one
for each file in tests/, and the examples to see that they compile) and runs them one after another, then
the doc tests. Most of its wall time is spent on the three or four longest programs while the machine's
other cores stand idle. This builds the same programs with cargo in the same way and runs them all at the
same time, each in the package's folder as cargo runs it, with the doc tests beside them. It ends with a
non-zero exit code if the build failed, or if any program failed, would not start or ran past its time
limit. Nothing is left out and nothing is added: the same tests run, and must pass, as under cargo test.

The profile is the dev one unless `--release` is given. The dev profile optimises the core as the release
one does (Cargo.toml) and adds debug assertions and overflow checks, so it checks more, not less; the two
golden tests give the same hashes under both. Its build is also the one the viewer's build makes anyway.

It prints one line for each program as it finishes (its seconds, its tests and its verdict), then the
whole output of each program that failed, then a last line: `tests: N programs, P passed, F failed, ...`.
Anything after `--` is handed to every test program (a name to filter by, `--nocapture`, ...).
Environment variables (ISOSTATE_GOLDEN_PRINT and the like) reach the programs as they would under cargo.
Written for the gate after the first step back (19 November 2026), which found the core's tests three
quarters of a landing; docs/rules/README.md, "Running the tests".
"""
import json, os, re, subprocess, sys, tempfile, time

HERE = os.path.dirname(os.path.abspath(__file__))


def main(argv):
    release, timeout, passed_on = False, 1800.0, []
    if "--" in argv:
        k = argv.index("--")
        argv, passed_on = argv[:k], argv[k + 1:]
    it = iter(argv)
    for a in it:
        if a == "--release":
            release = True
        elif a == "--timeout":
            timeout = float(next(it))
        elif a in ("-h", "--help"):
            print(__doc__)
            return 0
        else:
            print("unknown option %r\n%s" % (a, __doc__.splitlines()[2]), file=sys.stderr)
            return 2
    profile = ["--release"] if release else []
    t0 = time.time()
    # 1. the build, as cargo test does it (the examples are built too, so that one that no longer
    # compiles fails here as it fails there); cargo's own messages go to the terminal
    build = ["cargo", "test", "-p", "isostate-core", "--no-run", "--message-format=json-render-diagnostics"] + profile
    proc = subprocess.run(build, cwd=HERE, stdout=subprocess.PIPE, text=True, errors="replace")
    programs = []
    for line in proc.stdout.splitlines():
        try:
            m = json.loads(line)
        except ValueError:
            continue
        if m.get("reason") == "compiler-artifact" and m.get("profile", {}).get("test") and m.get("executable"):
            kind = m["target"]["kind"][0]
            name = m["target"]["name"] if kind == "test" else "%s (%s)" % (m["target"]["name"], kind)
            programs.append((name, [m["executable"]] + passed_on))
    if proc.returncode != 0:
        print("tests: the build failed (%s), so nothing was run" % " ".join(build), file=sys.stderr)
        return proc.returncode or 1
    if not programs:
        print("tests: the build gave no test programs", file=sys.stderr)
        return 1
    built = time.time() - t0
    # the doc tests, which cargo test runs last, as one more program beside the others
    programs.append(("doc tests", ["cargo", "test", "-p", "isostate-core", "--doc"] + profile + (["--"] + passed_on if passed_on else [])))
    print("built %d test programs in %.0f s; running them and the doc tests all at once" % (len(programs) - 1, built), flush=True)
    # 2. every program at once, each in the package's folder, as cargo runs it
    env = dict(os.environ, CARGO_MANIFEST_DIR=HERE, CARGO_PKG_NAME="isostate-core")
    logs = tempfile.mkdtemp(prefix="isostate-tests-")
    running = []
    for name, cmd in programs:
        log = open(os.path.join(logs, re.sub(r"[^A-Za-z0-9_.-]", "_", name) + ".log"), "w+", encoding="utf-8", errors="replace")
        try:
            p = subprocess.Popen(cmd, cwd=HERE, env=env, stdout=log, stderr=subprocess.STDOUT)
        except OSError as e:
            log.write("could not start %s: %s\n" % (cmd[0], e))
            p = None
        running.append((name, p, time.time(), log))
    done = []
    while running:
        for r in list(running):
            name, p, start, log = r
            rc = -1 if p is None else p.poll()
            if rc is None and time.time() - start > timeout:
                p.kill()
                p.wait()
                log.write("\n(stopped: still running after %.0f s, the time limit)\n" % timeout)
                rc = -2
            if rc is None:
                continue
            seconds = time.time() - start
            log.flush()
            log.seek(0)
            text = log.read()
            log.close()
            counts = [0, 0, 0]
            for m in re.finditer(r"^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored", text, flags=re.M):
                for k in range(3):
                    counts[k] += int(m.group(k + 1))
            ok = rc == 0
            print("  %-6s %6.1f s  %-24s %d passed, %d failed, %d ignored" % ("ok" if ok else "FAILED", seconds, name, *counts), flush=True)
            done.append((name, ok, rc, counts, text))
            running.remove(r)
        time.sleep(0.2)
    failed = [d for d in done if not d[1]]
    for name, _, rc, _, text in failed:
        why = {-1: "it would not start", -2: "it ran past the time limit"}.get(rc, "exit code %d" % rc)
        print("\n---- %s failed (%s); its output:\n%s" % (name, why, text.rstrip()))
    total = [sum(d[3][k] for d in done) for k in range(3)]
    end = time.time()
    print("\ntests: %d programs, %d passed, %d failed, %d ignored; %d program(s) failed; built in %.0f s, ran in %.0f s, %.0f s in all%s" % (
        len(done), total[0], total[1], total[2], len(failed), built, end - t0 - built, end - t0,
        "; their output is kept in " + logs if failed else ""), flush=True)
    if not failed:
        # the output is worth keeping only when something failed, and then it is printed above
        for f in os.listdir(logs):
            os.remove(os.path.join(logs, f))
        os.rmdir(logs)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
