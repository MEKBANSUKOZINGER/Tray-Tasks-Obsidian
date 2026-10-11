import { describe, it, expect } from "vitest";
import {
  addDays,
  escapeHtml,
  moveTaskGroup,
  reminderBounds,
} from "../src/utils";
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
describe("hierarchical display order", () => {
  const task = (id: string, parentId: string | null): Task => ({
    id,
    parentId,
    title: id,
    depth: parentId ? 1 : 0,
    done: false,
    reminderAt: null,
  });
  const tasks = [task("a", null), task("child", "a"), task("b", null)];
  it("moves the entire subtree without modifying markdown", () => {
    expect(moveTaskGroup(tasks, "b", "a")).toEqual(["b", "a", "child"]);
  });
  it("moves a parent below the next sibling together with its descendants", () => {
    expect(moveTaskGroup(tasks, "a", "b")).toEqual(["b", "a", "child"]);
  });
  it("does not detach a child from its parent", () => {
    expect(moveTaskGroup(tasks, "child", "b")).toEqual(["a", "child", "b"]);
  });
});
