import { pinyin } from "pinyin-pro";
import { ReadingProvider } from "./types";

export const zhProvider: ReadingProvider = {
  lang: "zh",
  annotate(text) {
    return pinyin(text)
  },
};