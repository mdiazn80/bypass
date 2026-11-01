import { describe, expect, it } from "vitest";
import { validateHosts, type OtherContext } from "./hostsValidation";

const lines = (content: string, others: OtherContext[] = [], position = 0) =>
  validateHosts(content, others, position).map((issue) => `${issue.line}: ${issue.message}`);

describe("validateHosts", () => {
  it("accepts valid IPv4 and IPv6 entries, comments and blank lines", () => {
    expect(
      lines(
        "# local\n\n127.0.0.1 a.local b.local # inline\n::1 localhost\nfe80::1%en0 link.local\n2001:db8::1 my_host.test."
      )
    ).toEqual([]);
  });

  it("flags invalid addresses", () => {
    expect(lines("256.0.0.1 a.local\n1.2.3 b.local\n2001:db8:::1 c.local\nlocalhost 1.2.3.4")).toEqual([
      '1: "256.0.0.1" is not a valid IPv4 or IPv6 address',
      '2: "1.2.3" is not a valid IPv4 or IPv6 address',
      '3: "2001:db8:::1" is not a valid IPv4 or IPv6 address',
      '4: "localhost" is not a valid IPv4 or IPv6 address',
    ]);
  });

  it("flags a line with no hostname and malformed hostnames", () => {
    expect(lines("127.0.0.1\n127.0.0.1 -bad.local *.local good.local")).toEqual([
      "1: No hostname after 127.0.0.1",
      '2: "-bad.local" is not a valid hostname',
      '2: "*.local": wildcards are not supported in hosts files',
    ]);
  });

  it("flags a hostname repeated within the context, ignoring case", () => {
    expect(lines("127.0.0.1 a.local\n10.0.0.1 A.LOCAL")).toEqual([
      "2: A.LOCAL is already defined on line 1, which wins",
    ]);
  });

  it("does not treat IPv4 and IPv6 entries for one name as duplicates", () => {
    expect(lines("127.0.0.1 a.local\n::1 a.local")).toEqual([]);
  });

  it("names the enabled context that defines the same host and who wins", () => {
    const others: OtherContext[] = [
      { name: "Prod", content: "10.0.0.1 api.local", enabled: true },
      { name: "Off", content: "10.0.0.2 api.local web.local", enabled: false },
      { name: "Later", content: "10.0.0.3 web.local", enabled: true },
    ];
    // This context sits between "Off" and "Later" in the sidebar.
    expect(lines("127.0.0.1 api.local web.local", others, 2)).toEqual([
      '1: api.local is also defined in "Prod", which comes first and wins',
      '1: web.local is also defined in "Later"; this context comes first and wins',
    ]);
  });
});
