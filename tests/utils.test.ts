import { describe, it, expect } from "vitest";
import { addDays, escapeHtml, moveTask, reminderBounds } from "../src/utils";
import type { Task } from "../src/types";
describe("calendar and safe rendering", () => {
  it("adds calendar days across leap years and year boundaries", () => {
    expect(addDays("2024-02-28", 1)).toBe("2024-02-29");
    expect(addDays("2026-12-31", 1)).toBe("2027-01-01");
    expect(addDays("2026-10-11", 7)).toBe("2026-10-18");
  });
  it("rounds reminder minimum up and ends tomorrow at 23:59", () => {
    expect(reminderBounds(new Date(2026, 9, 11, 23, 59, 35))).toEqual({
      min: "2026-10-12T00:00",
      max: "2026-10-12T23:59",
    });
  });
  it("never renders task text as executable markup", () => {
    expect(escapeHtml('<img src=x onerror="alert(1)">')).toBe(
      "&lt;img src=x onerror=&quot;alert(1)&quot;&gt;",
    );
  });
});
describe("independent display order", () => {
  const task = (id: string, parentId: string | null = null): Task => ({
    id,
    parentId,
    title: id,
    depth: 0,
    done: false,
    reminderAt: null,
  });
  const tasks = [task("a"), task("child", "a"), task("b")];
  it("moves only the selected checkbox even if markdown was indented", () => {
    expect(moveTask(tasks, "a", "b", true)).toEqual(["child", "b", "a"]);
    expect(moveTask(tasks, "child", "b", true)).toEqual(["a", "b", "child"]);
  });
  it("supports provisional placement before and after a target", () => {
    expect(moveTask(tasks, "b", "a")).toEqual(["b", "a", "child"]);
    expect(moveTask(tasks, "b", "a", true)).toEqual(["a", "b", "child"]);
  });
  it("ignores stale or identical identifiers without losing tasks", () => {
    expect(moveTask(tasks, "missing", "a")).toEqual(["a", "child", "b"]);
    expect(moveTask(tasks, "a", "a")).toEqual(["a", "child", "b"]);
  });
});
