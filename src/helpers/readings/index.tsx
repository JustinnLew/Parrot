import { ReadingProvider } from "./types";
import { zhProvider } from "./zh";


const providers: Record<string, ReadingProvider> = { zh: zhProvider };

export const getReadingProvider = (lang: string) =>
  providers[lang];