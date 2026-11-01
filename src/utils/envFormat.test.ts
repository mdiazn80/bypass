import { describe, expect, it } from "vitest";
import { parseEnv, parsePasted, safeFileName, toEnvFile } from "./envFormat";

describe("toEnvFile", () => {
  it("writes safe values bare, one per line, with a final newline", () => {
    expect(
      toEnvFile([
        { key: "HOST", value: "db.local:5432" },
        { key: "EMPTY", value: "" },
      ]),
    ).toBe("HOST=db.local:5432\nEMPTY=\n");
  });

  it("quotes values with spaces or shell-significant characters", () => {
    expect(toEnvFile([{ key: "GREETING", value: "hello world" }])).toBe('GREETING="hello world"\n');
    expect(toEnvFile([{ key: "REF", value: "{$TOKEN}" }])).toBe('REF="{$TOKEN}"\n');
  });

  it("escapes backslashes, quotes and newlines inside quoted values", () => {
    expect(toEnvFile([{ key: "V", value: 'a"b\\c\r\nd' }])).toBe('V="a\\"b\\\\c\\nd"\n');
  });
});

describe("parseEnv", () => {
  it("skips blank lines and comments and strips `export`", () => {
    expect(parseEnv("# comment\n\nexport A=1\r\nB = two \n")).toEqual([
      { key: "A", value: "1" },
      { key: "B", value: "two" },
    ]);
  });

  it("keeps everything after the first `=` as the value", () => {
    expect(parseEnv("URL=postgres://u:p@h/db?x=1")).toEqual([
      { key: "URL", value: "postgres://u:p@h/db?x=1" },
    ]);
  });

  it("removes matching single or double quotes only", () => {
    expect(parseEnv("A=\"x y\"\nB='z'\nC=\"open")).toEqual([
      { key: "A", value: "x y" },
      { key: "B", value: "z" },
      { key: "C", value: '"open' },
    ]);
  });

  it("turns a bare key into an empty value and drops lines with no key", () => {
    expect(parseEnv("ONLY_KEY\n=orphan")).toEqual([{ key: "ONLY_KEY", value: "" }]);
  });

  it("unescapes double-quoted values but keeps single-quoted ones literal", () => {
    expect(parseEnv('A="say \\"hi\\"\\nC:\\\\tmp"\nB=\'raw \\n\'')).toEqual([
      { key: "A", value: 'say "hi"\nC:\\tmp' },
      { key: "B", value: "raw \\n" },
    ]);
  });

  it("reads back exactly what toEnvFile writes", () => {
    const entries = [
      { key: "HOST", value: "db.local:5432" },
      { key: "GREETING", value: "hello world" },
      { key: "EMPTY", value: "" },
      { key: "QUOTED", value: 'a"b' },
      { key: "PATH_LIKE", value: "C:\\Users\\me" },
      { key: "MULTILINE", value: "line one\nline two" },
      { key: "LITERAL_ESCAPE", value: "not a \\n newline" },
      { key: "REF", value: "{$TOKEN}/v1" },
    ];
    expect(parseEnv(toEnvFile(entries))).toEqual(entries);
  });
});

describe("parsePasted", () => {
  it("ignores text with no KEY=value line", () => {
    expect(parsePasted("just some notes\n# and a comment")).toEqual([]);
  });

  it("parses text that has at least one assignment", () => {
    expect(parsePasted("notes\nTOKEN=abc")).toEqual([
      { key: "notes", value: "" },
      { key: "TOKEN", value: "abc" },
    ]);
  });
});

describe("safeFileName", () => {
  it("replaces anything outside letters, digits, `_` and `-`", () => {
    expect(safeFileName("Prod / EU-west 1.env")).toBe("Prod___EU-west_1_env");
  });
});
