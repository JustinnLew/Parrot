export interface ReadingProvider {
  lang: string;
  annotate(text: string): string;
}