// tts.ts
let voices: SpeechSynthesisVoice[] = [];
const refresh = () => (voices = speechSynthesis.getVoices());
refresh();
speechSynthesis.addEventListener("voiceschanged", refresh);

const BCP47: Record<string, string> = { en: "en-US", zh: "zh-CN" };

export function hasVoice(lang: string) {
  const tag = (BCP47[lang] ?? lang).toLowerCase();
  return voices.some((v) => v.lang.toLowerCase().startsWith(tag.slice(0, 2)));
}

export function speak(text: string, lang: string, rate = 0.9) {
  const tag = BCP47[lang] ?? lang;
  const u = new SpeechSynthesisUtterance(text);
  u.lang = tag;
  u.rate = rate;
  u.voice =
    voices.find((v) => v.lang === tag) ??
    voices.find((v) => v.lang.startsWith(tag.slice(0, 2))) ??
    null;
  speechSynthesis.cancel();
  speechSynthesis.speak(u);
}