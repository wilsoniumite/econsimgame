"""Make a marked version of the game: one zip a player can unpack and run on Windows.

    python tools/release.py              the version Cargo.toml says, from origin's main
    python tools/release.py --practice   the same from wherever the checkout stands, to try the
                                         command itself: its refusals become warnings

It refuses unless the checkout is clean, at origin's main, and its version not tagged yet.
Then it builds the game in release with no debugging information and with the machine's
folders taken out of the program (the `dist` profile, in target/dist), gathers the program and
what it needs into dist/isolated-state-VERSION-windows-x64.zip, unpacks the zip into an empty
folder away from the repository and has it take one picture there with nobody at the window,
looks for the builder's home folder and user name in the program and the zip, writes release
notes from the devlog, and prints the commands that would tag and publish the version. It runs
neither: tagging and publishing are done by hand (docs/RELEASING.md).

Python 3.11 or later, git and cargo; nothing else.
"""

import argparse
import datetime
import getpass
import hashlib
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import time
import tomllib
import zipfile
import zlib

# What goes into the zip beside the program: (the file in the repository, its place in the zip).
# The program carries its content, look, font and shaders inside it. The pages the game reads
# from disk as it runs are docs/BACKLOG.md, for the backlog card opened from the help card, and
# docs/DEVLOG.md, for the What's new card: crates/isostate-viewer/src/backlog_seen.rs looks for
# them in docs/ beside the program.
BESIDE = [
    ("docs/BACKLOG.md", "docs/BACKLOG.md"),
    ("docs/DEVLOG.md", "docs/DEVLOG.md"),
    ("LICENSE-MIT", "licences/LICENSE-MIT.txt"),
    ("LICENSE-APACHE", "licences/LICENSE-APACHE.txt"),
    ("crates/isostate-viewer/assets/fonts/OFL-Fredoka.txt", "licences/OFL-Fredoka.txt"),
]
PROGRAM = "isolated-state.exe"
TRIAL_ARGS = ["--sandbox", "--advance", "5", "--shot", "trial.png", "--size", "1600x1000"]
TRIAL_SECONDS = 900


def say(line=""):
    print(line, flush=True)


def run(args, cwd=None, env=None, check=True):
    out = subprocess.run(args, cwd=cwd, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace")
    if check and out.returncode != 0:
        raise SystemExit(f"failed: {' '.join(args)}\n{out.stdout}{out.stderr}")
    return out


def git(*args, check=True):
    return run(["git", *args], check=check).stdout.strip()


class Refusals:
    """A refusal stops a release, and is only a warning in a practice."""

    def __init__(self, practice):
        self.practice = practice
        self.waved = []

    def __call__(self, why):
        if self.practice:
            say(f"  (a practice, so only a warning) {why}")
            self.waved.append(why)
        else:
            raise SystemExit(f"refused: {why}\n(python tools/release.py --practice tries the command anyway, and publishes nothing)")


# ---------------------------------------------------------------- the checks before

def check_checkout(version, refuse):
    changed = git("status", "--porcelain", "--untracked-files=no")
    if changed:
        refuse("the checkout has changes not committed:\n" + changed)
    # files git does not know cannot reach the program (it is built from what the sources name),
    # so they are only listed
    loose = git("status", "--porcelain", "--untracked-files=normal")
    loose = [l for l in loose.splitlines() if l.startswith("??")]
    if loose:
        say(f"  {len(loose)} files git does not know are in the checkout; they cannot reach the program")
    # a practice asks nothing of origin and changes no ref: it goes by where origin's main was last seen
    if not refuse.practice:
        fetched = run(["git", "fetch", "--quiet", "--tags", "origin", "main"], check=False)
        if fetched.returncode != 0:
            refuse("origin could not be asked where main stands: " + fetched.stderr.strip())
    head = git("rev-parse", "HEAD")
    main = git("rev-parse", "--verify", "--quiet", "origin/main", check=False)
    if head != main:
        refuse(f"the checkout is at {head[:12]}, and origin's main at {main[:12] or 'nowhere known'}: a version is made from origin's main")
    if git("tag", "--list", f"v{version}"):
        refuse(f"v{version} is tagged already: raise the version in Cargo.toml (workspace.package), commit and push it, then make the version")


# ---------------------------------------------------------------- the build

def remapped_flags(root):
    """The machine's folders as rustc should write them in the program: the home folder as ~,
    cargo's and rustup's folders by their names, the repository as isolated-state. When two
    match, the one given last wins, so the home folder comes first."""
    home = os.path.expanduser("~")
    cargo = os.environ.get("CARGO_HOME") or os.path.join(home, ".cargo")
    rustup = os.environ.get("RUSTUP_HOME") or os.path.join(home, ".rustup")
    pairs = [(home, "~"), (cargo, "cargo"), (rustup, "rustup"), (root, "isolated-state")]
    return [f"--remap-path-prefix={os.path.normpath(p)}={to}" for p, to in pairs]


def build(root):
    env = dict(os.environ)
    env.pop("RUSTFLAGS", None)
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(remapped_flags(root))
    t = time.monotonic()
    out = subprocess.run(["cargo", "build", "--locked", "--profile", "dist", "-p", "isostate-viewer"], cwd=root, env=env)
    if out.returncode != 0:
        raise SystemExit("the build failed")
    return os.path.join(root, "target", "dist", "isostate-viewer.exe"), time.monotonic() - t


# ---------------------------------------------------------------- the zip

def third_party(root):
    """Every crate the program is built from, with its version and licence, from cargo's own
    account of the build (normal dependencies, for Windows)."""
    meta = json.loads(run(["cargo", "metadata", "--locked", "--format-version", "1", "--filter-platform", "x86_64-pc-windows-msvc"], cwd=root).stdout)
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    start = next(p["id"] for p in meta["packages"] if p["name"] == "isostate-viewer")
    seen, todo = set(), [start]
    while todo:
        n = todo.pop()
        if n in seen:
            continue
        seen.add(n)
        for d in nodes[n]["deps"]:
            if any(k["kind"] is None for k in d["dep_kinds"]):
                todo.append(d["pkg"])
    rows = sorted((packages[i]["name"], packages[i]["version"], packages[i].get("license") or "see its source", packages[i].get("repository") or "") for i in seen if packages[i]["source"])
    lines = [
        "The game is built with these crates, each under the licence it names (most under MIT or",
        "Apache-2.0, whose texts are in this folder). Each crate's copyright notices are in its source,",
        "at the address given or on crates.io.",
        "",
    ]
    lines += [f"{n} {v}  {lic}  {repo}".rstrip() for n, v, lic, repo in rows]
    return "\r\n".join(lines) + "\r\n", len(rows)


def player_readme(version, commit, day):
    text = f"""Isolated State {version}
(built on {day} from commit {commit})

An island's economy from 1750, played as a small diorama: its towns, works, roads and railways
grow as its people trade, and you watch it, change it by hand, or play one of its families.

TO PLAY
  Unpack the whole folder anywhere you like (the Desktop, Documents) and double-click
  {PROGRAM}. A black window of the game's messages opens beside it: leave it open, since closing
  it closes the game. Press H in the game for every key. The first start makes the island,
  which takes a few seconds.

  It opens on a start page: go on with a saved game, or begin a new island, as a sandbox
  (nobody plays a family, and the tray under the clock changes the island by hand) or as the
  family game. Its foot shows the command line that does the same, and {PROGRAM} --help
  lists everything else a command line can ask for.

WHAT IT NEEDS
  Windows 10 or 11, 64-bit, and graphics that do DirectX 12 or Vulkan (nearly every card or
  laptop since about 2015). Nothing else is installed: the program carries what it draws with.

WHEN WINDOWS WARNS YOU
  The program is not signed, so the first time it starts Windows may say "Windows protected your
  PC". Click "More info", then "Run anyway". It asks once for each new version.

WHERE YOUR GAMES ARE KEPT
  The game saves itself into %APPDATA%\\Isolated State\\saves (paste that into Explorer's
  address bar to open it). A note typed with N is kept there too, in the notes folder, with a
  picture of the screen. Saves and notes stay on the machine they were made on: to bring notes
  home, copy that notes folder. Deleting this game's folder leaves them where they are.

  The cards opened from the help card (H) read the docs folder here: the backlog, the plan of
  work as it stood when this version was made, and the devlog, what was done day by day.

WHAT IS IN THE FOLDER
  {PROGRAM}               the game
  docs\\BACKLOG.md         the plan of work, for the backlog card
  docs\\DEVLOG.md          what was done, newest first, for the What's new card
  licences\\               the game's licences (MIT or Apache-2.0, at your choice), the
                          font's (SIL Open Font Licence), and the crates it is built with
"""
    return text.replace("\n", "\r\n")


def make_zip(root, exe, version, commit, day, dist):
    name = f"isolated-state-{version}-windows-x64"
    path = os.path.join(dist, name + ".zip")
    top = f"isolated-state-{version}/"
    credits, crates = third_party(root)
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
        z.write(exe, top + PROGRAM)
        z.writestr(top + "README.txt", player_readme(version, commit, day))
        for src, at in BESIDE:
            z.write(os.path.join(root, src), top + at)
        z.writestr(top + "licences/crates.txt", credits)
    return path, top, crates


# ---------------------------------------------------------------- what the program must not carry

def views(data):
    """The bytes as text three ways: as they are, and every other byte from each of the first
    two, which reads text kept as UTF-16 (as Windows keeps paths) as plain letters."""
    return [data.decode("latin-1"), data[0::2].decode("latin-1"), data[1::2].decode("latin-1")]


def private_patterns():
    """The home folder (with either slash, and doubled backslashes as code writes them) and the
    user name as a word of its own; and git's name and address for this checkout, if set."""
    home = os.path.normpath(os.path.expanduser("~"))
    parts = [re.escape(p) for p in re.split(r"[\\/]", home) if p]
    found = [("the home folder", re.compile(r"[\\/]{1,2}".join(parts), re.I))]
    user = getpass.getuser()
    if user:
        found.append(("the user name", re.compile(rf"(?<![A-Za-z0-9]){re.escape(user)}(?![A-Za-z0-9])", re.I)))
    for key, what in [("user.name", "git's name"), ("user.email", "git's address")]:
        v = git("config", "--get", key, check=False)
        if v:
            found.append((what, re.compile(re.escape(v), re.I)))
    return found


def search(label, data, patterns):
    hits = []
    for text in views(data):
        for what, pat in patterns:
            for m in pat.finditer(text):
                a, b = max(0, m.start() - 30), min(len(text), m.end() + 30)
                hits.append(f"{label}: {what}, in ...{text[a:b]!r}...")
                break
    return hits


def search_all(exe, zpath, patterns):
    hits = search(os.path.basename(exe), open(exe, "rb").read(), patterns)
    hits += search(os.path.basename(zpath), open(zpath, "rb").read(), patterns)
    with zipfile.ZipFile(zpath) as z:
        for info in z.infolist():
            hits += search(info.filename, info.filename.encode() + z.read(info), patterns)
    return hits


# ---------------------------------------------------------------- the trial

def png_pixels(path):
    """Width, height and the rows of an 8-bit PNG, read with nothing but zlib."""
    data = open(path, "rb").read()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("not a PNG")
    pos, idat, w = 8, b"", 0
    while pos < len(data):
        n, kind = struct.unpack(">I4s", data[pos:pos + 8])
        body = data[pos + 8:pos + 8 + n]
        if kind == b"IHDR":
            w, h, depth, colour, _, _, interlace = struct.unpack(">IIBBBBB", body)
            if depth != 8 or interlace or colour not in (2, 6):
                raise ValueError(f"a PNG this check does not read (depth {depth}, colour {colour}, interlace {interlace})")
            bpp = 3 if colour == 2 else 4
        elif kind == b"IDAT":
            idat += body
        pos += 12 + n
    raw, stride, rows, prev = zlib.decompress(idat), w * bpp, [], bytearray(w * bpp)
    for y in range(h):
        f, line = raw[y * (stride + 1)], bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for i in range(stride):
            a = line[i - bpp] if i >= bpp else 0
            b, c = prev[i], prev[i - bpp] if i >= bpp else 0
            if f == 1:
                line[i] = (line[i] + a) & 255
            elif f == 2:
                line[i] = (line[i] + b) & 255
            elif f == 3:
                line[i] = (line[i] + (a + b) // 2) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                line[i] = (line[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(line))
        prev = line
    return w, h, bpp, rows


def not_blank(path):
    """A picture is not blank when its colours are many and no one of them covers most of it."""
    w, h, bpp, rows = png_pixels(path)
    counts = {}
    for y in range(0, h, 4):
        row = rows[y]
        for x in range(0, w, 4):
            px = row[x * bpp:x * bpp + 3]
            counts[px] = counts.get(px, 0) + 1
    total = sum(counts.values())
    commonest = max(counts.values()) / total
    return len(counts) >= 200 and commonest < 0.5, f"{w} by {h} pixels, {len(counts)} colours among {total} sampled, the commonest on {commonest:.0%} of them"


def reaches_repository(folder):
    """Whether the game started in `folder` could find the repository: a folder above it that
    holds the repository's Cargo.toml, .git or docs/BACKLOG.md."""
    d = os.path.dirname(folder)
    while True:
        if any(os.path.exists(os.path.join(d, x)) for x in ("Cargo.toml", ".git", os.path.join("docs", "BACKLOG.md"))):
            return d
        up = os.path.dirname(d)
        if up == d:
            return None
        d = up


def trial(zpath, top, version, commit, root):
    """Unpack the zip into an empty folder away from the repository, and have the game take one
    picture there with nobody at the window."""
    where = tempfile.mkdtemp(prefix="isolated-state-trial-")
    near = reaches_repository(where)
    if near or os.path.normcase(where).startswith(os.path.normcase(root)):
        raise SystemExit(f"the trial's folder {where} is within reach of a repository at {near or root}: set TEMP to a folder outside it")
    with zipfile.ZipFile(zpath) as z:
        z.extractall(where)
    game = os.path.join(where, top.rstrip("/"))
    exe = os.path.join(game, PROGRAM)
    # nothing of the build or of the repository in its surroundings, and its own saves' folder
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith(("CARGO", "RUSTUP", "RUSTFLAGS", "BEVY_", "ISOSTATE_", "WGPU_"))}
    env["PATH"] = os.pathsep.join(p for p in env.get("PATH", "").split(os.pathsep) if p and not os.path.normcase(os.path.abspath(p)).startswith(os.path.normcase(root)))
    env["ISOSTATE_SAVES"] = os.path.join(where, "saves")
    said = run([exe, "--version"], cwd=game, env=env).stdout.strip()
    if version not in said or commit not in said:
        raise SystemExit(f"the unpacked program says {said!r}, where {version} and {commit} were looked for")
    say(f"  the unpacked program says: {said}")
    t = time.monotonic()
    try:
        out = subprocess.run([exe, *TRIAL_ARGS], cwd=game, env=env, capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=TRIAL_SECONDS)
    except subprocess.TimeoutExpired:
        raise SystemExit(f"the trial took more than {TRIAL_SECONDS} s; its folder is {where}")
    took = time.monotonic() - t
    picture = os.path.join(game, "trial.png")
    log = os.path.join(where, "trial.log")
    with open(log, "w", encoding="utf-8") as f:
        f.write(out.stdout + out.stderr)
    if out.returncode != 0 or not os.path.isfile(picture):
        raise SystemExit(f"the trial ended with code {out.returncode} and {'a' if os.path.isfile(picture) else 'no'} picture; what it said is in {log}:\n{(out.stdout + out.stderr)[-3000:]}")
    ok, how = not_blank(picture)
    if not ok:
        raise SystemExit(f"the trial's picture looks blank ({how}): {picture}")
    return picture, how, took, where


# ---------------------------------------------------------------- release notes

ENTRY = re.compile(r"^\*\*(.+?)\*\*\s*(.*)$")
DAY = re.compile(r"^## (\d{4}-\d{2}-\d{2})")
# words that say an entry is something a player sees or does, and the sentence that says where
IN_THE_GAME = re.compile(r"\b(in the viewer|the viewer|tab|panel|card|chip|map views?|button|tray|toast|the help card|key|press|click|inspector|the bar|on screen)\b", re.I)


def devlog_entries(text):
    """The devlog's entries, newest first: (day, title, the rest of the paragraph)."""
    day, found = None, []
    for line in text.splitlines():
        m = DAY.match(line)
        if m:
            day = m.group(1)
            continue
        m = ENTRY.match(line.strip())
        if m and day:
            title = m.group(1).strip().rstrip(".:")
            found.append((day, title, m.group(2)))
    return found


def first_sentence_with(pattern, text):
    for s in re.split(r"(?<=[.!?])\s+", text):
        if pattern.search(s):
            return s.strip()
    return None


def release_notes(version, commit, day):
    now = devlog_entries(open(os.path.join("docs", "DEVLOG.md"), encoding="utf-8").read())
    last = git("describe", "--tags", "--abbrev=0", "--match", "v*", "HEAD", check=False)
    if last:
        before = git("show", f"{last}:docs/DEVLOG.md", check=False)
        known = {t for _, t, _ in devlog_entries(before)}
        new = [e for e in now if e[1] not in known]
        since = f"since {last}"
    else:
        # the first marked version: the devlog's last seven days
        newest = datetime.date.fromisoformat(now[0][0]) if now else datetime.date.today()
        new = [e for e in now if (newest - datetime.date.fromisoformat(e[0])).days < 7]
        since = f"in the devlog's last seven days, to {newest.isoformat()} (the first marked version)"
    seen, game, model = set(), [], []
    for d, title, rest in new:
        if title in seen:
            continue
        seen.add(title)
        where = first_sentence_with(IN_THE_GAME, rest)
        if where or IN_THE_GAME.search(title):
            game.append((title, where or ""))
        else:
            model.append((title, None))
    lines = [
        f"# Isolated State {version}",
        "",
        f"Built from commit {commit} on {day}. Windows 10 or 11, 64-bit: unpack the zip anywhere and run isolated-state.exe (README.txt inside says more).",
        "",
        f"What is new {since}, from the devlog's headings. Written by tools/release.py: read it through and shorten it before publishing.",
        "",
    ]
    if game:
        lines += ["## In the game", ""]
        lines += [f"- **{t}.** {w}".rstrip() for t, w in game]
        lines.append("")
    if model:
        lines += ["## In the island's economy", ""]
        lines += [f"- {t}." for t, _ in model]
        lines.append("")
    return "\n".join(lines), len(game) + len(model), since


# ---------------------------------------------------------------- the command

def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--practice", action="store_true", help="try the whole command from wherever the checkout stands; its refusals become warnings")
    ap.add_argument("--keep-trial", action="store_true", help="leave the trial's folder where it is, to look at")
    a = ap.parse_args()
    t0 = time.monotonic()
    root = os.path.normpath(git("rev-parse", "--show-toplevel"))
    os.chdir(root)
    version = tomllib.load(open("Cargo.toml", "rb"))["workspace"]["package"]["version"]
    commit = git("rev-parse", "--short=12", "HEAD")
    day = datetime.date.today().isoformat()
    say(f"Isolated State {version}, from commit {commit}{' (a practice)' if a.practice else ''}")
    refuse = Refusals(a.practice)

    say("1. the checkout")
    check_checkout(version, refuse)

    say("2. the build (cargo build --profile dist -p isostate-viewer, with the machine's folders taken out)")
    exe, build_s = build(root)
    said = run([exe, "--version"]).stdout.strip()
    say(f"  built in {build_s:.0f} s: {os.path.getsize(exe) / 2**20:.1f} MiB; it says: {said}")
    if version not in said or commit not in said:
        refuse(f"the program says {said!r}, where {version} and {commit} were looked for")

    say("3. the zip")
    dist = os.path.join(root, "dist")
    os.makedirs(dist, exist_ok=True)
    zpath, top, crates = make_zip(root, exe, version, commit, day, dist)
    size = os.path.getsize(zpath)
    sha = hashlib.sha256(open(zpath, "rb").read()).hexdigest()
    say(f"  {os.path.relpath(zpath)}: {size / 2**20:.1f} MiB, with the {crates} crates' licences listed")

    say("4. what the program and the zip must not carry")
    hits = search_all(exe, zpath, private_patterns())
    if hits:
        raise SystemExit("found in the program or the zip:\n  " + "\n  ".join(hits[:20]))
    say("  neither the home folder, the user name nor git's name or address is in the program or the zip")

    say("5. the trial: the zip unpacked into an empty folder away from the repository, and one picture taken there")
    picture, how, trial_s, where = trial(zpath, top, version, commit, root)
    kept = os.path.join(dist, f"isolated-state-{version}-trial.png")
    shutil.copyfile(picture, kept)
    say(f"  started and took its picture in {trial_s:.0f} s: {how}")
    say(f"  the picture is kept as {os.path.relpath(kept)}")
    if a.keep_trial:
        say(f"  the trial's folder is kept: {where}")
    else:
        shutil.rmtree(where, ignore_errors=True)

    say("6. the release notes")
    notes, count, since = release_notes(version, commit, day)
    npath = os.path.join(dist, f"isolated-state-{version}-notes.md")
    with open(npath, "w", encoding="utf-8") as f:
        f.write(notes)
    say(f"  {os.path.relpath(npath)}: {count} entries {since}")

    total = time.monotonic() - t0
    say()
    say(f"Made in {total / 60:.1f} minutes (the build {build_s / 60:.1f}): {os.path.relpath(zpath)}, {size / 2**20:.1f} MiB, sha256 {sha}")
    say()
    if refuse.waved:
        say("This was a practice, and these would have refused a release:")
        for w in refuse.waved:
            say(f"  - {w.splitlines()[0]}")
        say("Nothing below is to be run for it.")
        say()
    say("To mark and publish the version (nothing has been run; read the notes through first):")
    tag = f"v{version}"
    say(f'  git tag -a {tag} -m "Isolated State {version}" {git("rev-parse", "HEAD")}')
    say(f"  git push origin {tag}")
    say(f'  gh release create {tag} "{os.path.relpath(zpath)}" --title "Isolated State {version}" --notes-file "{os.path.relpath(npath)}" --prerelease')
    origin = re.sub(r"\.git$", "", git("remote", "get-url", "origin", check=False))
    m = re.search(r"github\.com[:/](.+)$", origin)
    if m:
        say(f"Then it is at https://github.com/{m.group(1)}/releases/latest, to download on any machine.")


if __name__ == "__main__":
    main()
