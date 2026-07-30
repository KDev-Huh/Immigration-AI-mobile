import type { ReactNode } from "react";

interface Props {
  content: string;
}

const inlineRe = /(`[^`]+`|\*\*[^*]+\*\*|\[[^\]]+\]\(https?:\/\/[^\s)]+\))/g;

function parseInline(text: string): ReactNode[] {
  const out: ReactNode[] = [];
  let last = 0;

  for (const match of text.matchAll(inlineRe)) {
    const token = match[0];
    const start = match.index ?? 0;
    if (start > last) out.push(text.slice(last, start));

    if (token.startsWith("`")) {
      out.push(<code key={start}>{token.slice(1, -1)}</code>);
    } else if (token.startsWith("**")) {
      out.push(<strong key={start}>{token.slice(2, -2)}</strong>);
    } else {
      const link = token.match(/^\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)$/);
      out.push(
        link ? (
          <a key={start} href={link[2]} target="_blank" rel="noreferrer">
            {link[1]}
          </a>
        ) : (
          token
        ),
      );
    }
    last = start + token.length;
  }

  if (last < text.length) out.push(text.slice(last));
  return out;
}

function parseInlineWithBreaks(text: string): ReactNode[] {
  return text.split("\n").flatMap((line, i) =>
    i === 0 ? parseInline(line) : [<br key={`br-${i}`} />, ...parseInline(line)],
  );
}

const isHeading = (line: string) => /^#{1,3}\s+/.test(line);
const isUnordered = (line: string) => /^\s*[-*]\s+/.test(line);
const isOrdered = (line: string) => /^\s*\d+[.)]\s+/.test(line);
const isQuote = (line: string) => /^\s*>\s?/.test(line);
const isBlockStart = (line: string) =>
  line.startsWith("```") ||
  isHeading(line) ||
  isUnordered(line) ||
  isOrdered(line) ||
  isQuote(line);

export default function MarkdownMessage({ content }: Props) {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  const blocks: ReactNode[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i += 1;
      continue;
    }

    if (line.startsWith("```")) {
      const lang = line.slice(3).trim();
      const code: string[] = [];
      i += 1;
      while (i < lines.length && !lines[i].startsWith("```")) {
        code.push(lines[i]);
        i += 1;
      }
      if (i < lines.length) i += 1;
      blocks.push(
        <pre key={blocks.length}>
          <code data-lang={lang || undefined}>{code.join("\n")}</code>
        </pre>,
      );
      continue;
    }

    const heading = line.match(/^(#{1,3})\s+(.+)$/);
    if (heading) {
      const level = heading[1].length;
      const body = parseInline(heading[2]);
      blocks.push(
        level === 1 ? (
          <h3 key={blocks.length}>{body}</h3>
        ) : level === 2 ? (
          <h4 key={blocks.length}>{body}</h4>
        ) : (
          <h5 key={blocks.length}>{body}</h5>
        ),
      );
      i += 1;
      continue;
    }

    if (isUnordered(line) || isOrdered(line)) {
      const ordered = isOrdered(line);
      const items: ReactNode[] = [];
      while (
        i < lines.length &&
        (ordered ? isOrdered(lines[i]) : isUnordered(lines[i]))
      ) {
        items.push(
          <li key={items.length}>
            {parseInline(lines[i].replace(ordered ? /^\s*\d+[.)]\s+/ : /^\s*[-*]\s+/, ""))}
          </li>,
        );
        i += 1;
      }
      blocks.push(
        ordered ? (
          <ol key={blocks.length}>{items}</ol>
        ) : (
          <ul key={blocks.length}>{items}</ul>
        ),
      );
      continue;
    }

    if (isQuote(line)) {
      const quote: string[] = [];
      while (i < lines.length && isQuote(lines[i])) {
        quote.push(lines[i].replace(/^\s*>\s?/, ""));
        i += 1;
      }
      blocks.push(
        <blockquote key={blocks.length}>
          {parseInlineWithBreaks(quote.join("\n"))}
        </blockquote>,
      );
      continue;
    }

    const para: string[] = [];
    while (i < lines.length && lines[i].trim() && (para.length === 0 || !isBlockStart(lines[i]))) {
      para.push(lines[i]);
      i += 1;
    }
    blocks.push(<p key={blocks.length}>{parseInlineWithBreaks(para.join("\n"))}</p>);
  }

  return <div className="markdown">{blocks}</div>;
}
