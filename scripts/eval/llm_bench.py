"""Benchmark of text normalizers: tn rules, LLMs served by an
OpenAI-compatible endpoint (zallama, llama-server), and rules + LLM.

Usage:
  python3 scripts/eval/llm_bench.py <base_url> <system> [<system> ...]

Systems:
  @rules              tn strict mode (TN_BIN, default target/release/tn-server)
  @safe               tn safe mode
  <model>             the LLM alone (few-shot prompt below, temperature 0)
  @hybrid:<model>     tn safe mode, then the LLM on sentences with leftovers

Scores: WER against hand-written references (min over accepted variants),
plain input words missing from the output (content the normalizer must never
change), latency. VRAM is read from nvidia-smi for a llama-server whose
command line contains the file named in TN_BENCH_FILES (JSON model -> file).
Results of 2026-10-02: docs/llm-pass.md.
"""
import os
import json, re, sys, time, unicodedata, urllib.request

SYS = (
    "You are a text normalizer for a text-to-speech engine. Rewrite the user's text exactly as it "
    "should be spoken aloud, in the same language: spell out numbers, dates, times, amounts with "
    "currency, percentages, temperatures, units, ordinals, Roman numerals, phone numbers and "
    "abbreviations; remove Markdown symbols. Keep every other word, in the same order, with the same "
    "punctuation. Output only the rewritten text."
)
SHOTS = [
    ("Le train part à 7h45 du quai 3, billet à 23,50 €.",
     "Le train part à sept heures quarante-cinq du quai trois, billet à vingt-trois euros cinquante."),
    ("The 2nd edition sold 15,000 copies in 2019 at $9.99 each.",
     "The second edition sold fifteen thousand copies in twenty nineteen at nine dollars and ninety-nine cents each."),
    ("**Rappel** : M. Leroy arrive le 12/03 à 16h.",
     "Rappel : Monsieur Leroy arrive le douze mars à seize heures."),
]

CASES = [
    ("La réunion commence à 9h30 et le budget atteint 1 250 000 €.",
     ["La réunion commence à neuf heures trente et le budget atteint un million deux cent cinquante mille euros."]),
    ("Le 1er mai, il fera entre 18 et 24 °C, soit 3,5 % de plus.",
     ["Le premier mai, il fera entre dix-huit et vingt-quatre degrés, soit trois virgule cinq pour cent de plus.",
      "Le premier mai, il fera entre dix-huit et vingt-quatre degrés Celsius, soit trois virgule cinq pour cent de plus."]),
    ("Appelez le 06 12 34 56 78 avant le 21/10/2026, Mme Dupont.",
     ["Appelez le zéro six douze trente-quatre cinquante-six soixante-dix-huit avant le vingt et un octobre deux mille vingt-six, Madame Dupont."]),
    ("Au XVIIe siècle, Louis XIV régnait déjà depuis 1643.",
     ["Au dix-septième siècle, Louis quatorze régnait déjà depuis mille six cent quarante-trois.",
      "Au dix-septième siècle, Louis quatorze régnait déjà depuis seize cent quarante-trois."]),
    ("Le colis pèse 2,5 kg et coûte 12,99 €.",
     ["Le colis pèse deux virgule cinq kilos et coûte douze euros quatre-vingt-dix-neuf.",
      "Le colis pèse deux virgule cinq kilogrammes et coûte douze euros quatre-vingt-dix-neuf.",
      "Le colis pèse deux kilos et demi et coûte douze euros quatre-vingt-dix-neuf."]),
    ("Rendez-vous au 3e étage, salle n° 204, à 14h15.",
     ["Rendez-vous au troisième étage, salle numéro deux cent quatre, à quatorze heures quinze."]),
    ("**Important** : la vitesse est limitée à 80 km/h depuis 2018.",
     ["Important : la vitesse est limitée à quatre-vingts kilomètres heure depuis deux mille dix-huit.",
      "Important : la vitesse est limitée à quatre-vingts kilomètres par heure depuis deux mille dix-huit.",
      "Important : la vitesse est limitée à quatre-vingts kilomètres à l'heure depuis deux mille dix-huit."]),
    ("Le Dr Martin reçoit du lundi au vendredi, de 8h à 18h30.",
     ["Le docteur Martin reçoit du lundi au vendredi, de huit heures à dix-huit heures trente."]),
    ("Meet at 9:30 am, it costs $5.50 and rose 12% in 1984.",
     ["Meet at nine thirty am, it costs five dollars and fifty cents and rose twelve percent in nineteen eighty-four.",
      "Meet at nine thirty am, it costs five dollars fifty and rose twelve percent in nineteen eighty-four."]),
    ("Dr. Smith lives at 221B Baker St., call (555) 123-4567 on Oct. 21, 2026.",
     ["Doctor Smith lives at two twenty-one B Baker Street, call five five five, one two three, four five six seven on October twenty-first, twenty twenty-six.",
      "Doctor Smith lives at two hundred twenty-one B Baker Street, call five five five, one two three, four five six seven on October twenty-first, twenty twenty-six."]),
    ("The 3rd quarter revenue was $2.4 million, up 7.5% from Q2.",
     ["The third quarter revenue was two point four million dollars, up seven point five percent from Q two."]),
    ("It weighs 3.2 kg and measures 45 cm.",
     ["It weighs three point two kilograms and measures forty-five centimeters."]),
    ("World War II ended in 1945; Henry VIII died in 1547.",
     ["World War two ended in nineteen forty-five; Henry the eighth died in fifteen forty-seven.",
      "World War two ended in nineteen forty-five; Henry the Eighth died in fifteen forty-seven."]),
    ("The flight leaves at 6:05 pm from gate 12B.",
     ["The flight leaves at six oh five pm from gate twelve B."]),
    ("**Note:** the temperature dropped to -5°C overnight.",
     ["Note: the temperature dropped to minus five degrees Celsius overnight.",
      "Note: the temperature dropped to minus five degrees overnight."]),
    ("Call 1-800-555-0199 before Jan. 1st.",
     ["Call one eight hundred, five five five, zero one nine nine before January first.",
      "Call one eight zero zero, five five five, zero one nine nine before January first."]),
]

def words(s):
    s = unicodedata.normalize("NFC", s.lower())
    s = re.sub(r"\b([ap])\.?\s?m\b\.?", r"\1m", s)
    s = s.replace("’", "'").replace("-", " ")
    s = re.sub(r"[^\w' ]+", " ", s)
    return s.split()

def wer(hyp, ref):
    h, r = words(hyp), words(ref)
    d = list(range(len(h) + 1))
    for i, rw in enumerate(r, 1):
        prev, d[0] = d[0], i
        for j, hw in enumerate(h, 1):
            cur = min(d[j] + 1, d[j - 1] + 1, prev + (rw != hw))
            prev, d[j] = d[j], cur
    return d[len(h)] / max(1, len(r))

ABBR = {"mme", "dr", "st", "oct", "jan", "q2", "km", "kg", "cm", "am", "pm", "xvie", "xviie", "xiv", "ii", "viii", "n"}

def dropped(inp, out):
    """Plain words of the input missing from the output (content changes)."""
    o = set(words(out))
    return [w for w in words(inp)
            if w.isalpha() and len(w) > 1 and w not in ABBR and not re.search(r"\d", w) and w not in o]

NEEDS_LLM = re.compile(r"[0-9]|\b[IVXLC]{2,}\b|\b[IVXLC]+e\b|\b[A-Z][a-z]{1,3}\.")


def ask(base, model, text):
    if model.startswith("@hybrid:"):
        ruled = ask(base, "@safe", text)
        return ask(base, model.split(":", 1)[1], ruled) if NEEDS_LLM.search(ruled) else ruled
    if model in ("@rules", "@safe"):
        import subprocess
        idx = next((i for i, c in enumerate(CASES) if c[0] == text), 99)
        if idx == 99:  # hybrid passes rules output of a known case: language by accents/words
            idx = 0 if re.search(r"[éèàùç]|\b(le|la|les|au|du|de)\b", text) else 99
        lang = "fr" if idx < 8 else "en"
        flag = ["--safe"] if model == "@safe" else []
        tn = os.environ.get("TN_BIN", "target/release/tn-server")
        return subprocess.run([tn, "normalize", *flag, "--lang", lang, text],
                              capture_output=True, text=True).stdout.strip()
    msgs = [{"role": "system", "content": SYS}]
    for i, o in SHOTS:
        msgs += [{"role": "user", "content": i}, {"role": "assistant", "content": o}]
    msgs.append({"role": "user", "content": text})
    body = {"model": model, "temperature": 0, "max_tokens": 400, "messages": msgs,
            "chat_template_kwargs": {"enable_thinking": False}}
    req = urllib.request.Request(base + "/v1/chat/completions", json.dumps(body).encode(),
                                 {"content-type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=600))["choices"][0]["message"]["content"].strip()

def vram(model):
    """VRAM of the llama-server whose command line names this model's file."""
    import subprocess
    if model.startswith("@"):
        return "0"
    apps = subprocess.run(["nvidia-smi", "--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"],
                          capture_output=True, text=True).stdout
    for line in apps.splitlines():
        pid, mib = [x.strip() for x in line.split(",")]
        try:
            cmd = open(f"/proc/{pid}/cmdline").read()
        except OSError:
            continue
        if FILES.get(model, "\0") in cmd:
            return f"{int(mib)/1024:.2f} GB"
    return "?"


FILES = json.load(open(os.environ["TN_BENCH_FILES"])) if os.environ.get("TN_BENCH_FILES") else {}


def main():
    base, models = sys.argv[1], sys.argv[2:]
    for model in models:
        try:
            ask(base, model, "Test 1.")  # load + warm up
        except Exception as e:
            print(f"== {model}: FAILED to load ({e})"); continue
        tot_w, tot_d, tot_t, rows = 0.0, 0, 0.0, []
        for inp, refs in CASES:
            t0 = time.time()
            out = ask(base, model, inp)
            dt = time.time() - t0
            w = min(wer(out, r) for r in refs)
            d = dropped(inp, out)
            tot_w += w; tot_d += len(d); tot_t += dt
            rows.append((w, d, dt, inp, out))
        n = len(CASES)
        print(f"== {model}: WER {100*tot_w/n:.1f}%  dropped words {tot_d}  {1000*tot_t/n:.0f} ms/sentence  VRAM {vram(model)}")
        for w, d, dt, inp, out in rows:
            flag = "OK " if w == 0 and not d else "   "
            print(f"  {flag}{100*w:5.1f}% {('DROP ' + ','.join(d)) if d else ''}\n      {out}")
        sys.stdout.flush()

if __name__ == "__main__":
    main()
