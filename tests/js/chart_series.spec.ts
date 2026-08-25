import assert from "node:assert/strict";
import { createRequire } from "node:module";
import test from "node:test";

const browser_globals_apexcharts_reads_when_it_loads = {
  document: { body: null },
  add_init_fn: () => {},
};
Object.assign(globalThis, browser_globals_apexcharts_reads_when_it_loads);

const require = createRequire(import.meta.url);
const {
  align_series,
  align_series_for,
  merged_x_values,
} = require("../../sqlpage/apexcharts.js");

const ADDS_NOTHING_TO_THE_STACK = 0;
const LEAVES_A_GAP = null;
const STACKED = true;
const UNSTACKED = false;

type XValue = number | string | Date;
type Point = {
  x: XValue;
  y: number | string | null | number[];
  z?: number;
  fillColor?: string;
};
type Series = { name: string; data: Point[] };

const series = (name: string, ...data: Point[]): Series => ({ name, data });
const xs = (s: Series) => s.data.map((p) => p.x);

test("merged_x_values keeps the order the series agree on", () => {
  const merged = merged_x_values([
    series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }, { x: "Q3", y: 3 }),
    series("b", { x: "Q1", y: 4 }, { x: "Q2", y: 5 }, { x: "Q3", y: 6 }),
  ]);

  assert.deepEqual(merged, ["Q1", "Q2", "Q3"]);
});

test("merged_x_values orders by name the x values the series do not share (#951)", () => {
  const merged = merged_x_values([
    series("a", { x: "X2", y: 10 }, { x: "X3", y: 30 }),
    series("b", { x: "X1", y: 25 }, { x: "X2", y: 20 }),
  ]);

  assert.deepEqual(merged, ["X1", "X2", "X3"]);
});

test("merged_x_values compares numbers as numbers, not as text", () => {
  const merged = merged_x_values([
    series("a", { x: 2, y: 1 }, { x: 10, y: 1 }),
    series("b", { x: 9, y: 1 }),
  ]);

  assert.deepEqual(merged, [2, 9, 10]);
});

test("merged_x_values matches equal dates written as different objects", () => {
  const merged = merged_x_values([
    series("a", { x: new Date("2024-03-01"), y: 1 }),
    series("b", { x: new Date("2024-03-01"), y: 2 }),
  ]);

  assert.equal(merged.length, 1);
});

test("merged_x_values ignores series that hold no points", () => {
  const merged = merged_x_values([
    series("empty"),
    series("a", { x: 1, y: 1 }),
  ]);

  assert.deepEqual(merged, [1]);
});

test("align_series gives every series a point at every x (#727)", () => {
  const [a, b] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 3 }),
    ],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(xs(a), ["Q1", "Q2"]);
  assert.deepEqual(xs(b), ["Q1", "Q2"]);
});

test("align_series counts an x a stacked series skipped as zero (#727)", () => {
  const [, b] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 3 }),
    ],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(b.data, [
    { x: "Q1", y: 0 },
    { x: "Q2", y: 3 },
  ]);
});

test("align_series leaves a gap where an unstacked series has no value", () => {
  const [, b] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 3 }),
    ],
    LEAVES_A_GAP,
  );

  assert.deepEqual(b.data, [
    { x: "Q1", y: null },
    { x: "Q2", y: 3 },
  ]);
});

test("align_series keeps a measured zero apart from a missing value", () => {
  const [, b] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 0 }),
    ],
    LEAVES_A_GAP,
  );

  assert.deepEqual(b.data, [
    { x: "Q1", y: null },
    { x: "Q2", y: 0 },
  ]);
});

test("align_series counts a null value as missing", () => {
  const [, b] = align_series(
    [series("a", { x: "Q1", y: 1 }), series("b", { x: "Q1", y: null })],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(b.data, [{ x: "Q1", y: 0 }]);
});

test("align_series keeps a blank value the series wrote", () => {
  const [, b] = align_series(
    [series("a", { x: "Q1", y: 1 }), series("b", { x: "Q1", y: "" })],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(b.data, [{ x: "Q1", y: "" }]);
});

test("align_series keeps a value the series wrote as text", () => {
  const [, b] = align_series(
    [series("a", { x: "Q1", y: 1 }), series("b", { x: "Q1", y: "7" })],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(b.data, [{ x: "Q1", y: "7" }]);
});

test("align_series keeps the third dimension of points it did not fill in", () => {
  const [a] = align_series(
    [series("a", { x: "Q1", y: 1, z: 42 }), series("b", { x: "Q2", y: 2 })],
    LEAVES_A_GAP,
  );

  assert.equal(a.data[0].z, 42);
});

test("align_series keeps the color of a point on the x it belongs to", () => {
  const [, b] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 20, fillColor: "#37b24d" }),
    ],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(
    b.data.map((point) => point.fillColor),
    [undefined, "#37b24d"],
  );
});

test("align_series matches dates by value rather than by identity", () => {
  const [a, b] = align_series(
    [
      series("a", { x: new Date("2024-03-01"), y: 1 }),
      series("b", { x: new Date("2024-03-01"), y: 2 }),
    ],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.equal(a.data.length, 1);
  assert.equal(b.data.length, 1);
  assert.equal(b.data[0].y, 2);
});

test("align_series leaves a lone series in the order it arrived (#930)", () => {
  const [only] = align_series(
    [series("a", { x: "Q2", y: 1 }, { x: "Q1", y: 2 })],
    LEAVES_A_GAP,
  );

  assert.deepEqual(only.data, [
    { x: "Q2", y: 1 },
    { x: "Q1", y: 2 },
  ]);
});

test("align_series returns series that already share every x unchanged", () => {
  const given = [
    series("a", { x: "Q3", y: 1 }, { x: "Q1", y: 2 }, { x: "Q2", y: 3 }),
    series("b", { x: "Q3", y: 4 }, { x: "Q1", y: 5 }, { x: "Q2", y: 6 }),
  ];

  assert.deepEqual(align_series(given, ADDS_NOTHING_TO_THE_STACK), given);
});

test("align_series does not mutate the series it is given", () => {
  const given = [
    series("a", { x: "Q1", y: 1 }),
    series("b", { x: "Q2", y: 2 }),
  ];
  const before = JSON.stringify(given);

  align_series(given, LEAVES_A_GAP);

  assert.equal(JSON.stringify(given), before);
});

test("align_series keeps the last of duplicated x values", () => {
  const [a] = align_series(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q1", y: 9 }),
      series("b", { x: "Q2", y: 2 }),
    ],
    ADDS_NOTHING_TO_THE_STACK,
  );

  assert.deepEqual(a.data, [
    { x: "Q1", y: 9 },
    { x: "Q2", y: 0 },
  ]);
});

test("align_series_for gives a stacked series a zero at every x it skipped", () => {
  const [, b] = align_series_for(
    [series("a", { x: 1, y: 1 }, { x: 2, y: 2 }), series("b", { x: 2, y: 3 })],
    "area",
    STACKED,
  );

  assert.deepEqual(b.data, [
    { x: 1, y: 0 },
    { x: 2, y: 3 },
  ]);
});

for (const type of ["line", "area", "scatter", "bubble", "heatmap"]) {
  test(`align_series_for leaves a gap where a ${type} series skips a label`, () => {
    const [, b] = align_series_for(
      [
        series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
        series("b", { x: "Q2", y: 3 }),
      ],
      type,
      UNSTACKED,
    );

    assert.deepEqual(b.data, [
      { x: "Q1", y: null },
      { x: "Q2", y: 3 },
    ]);
  });
}

test("align_series_for counts a label a bar series skipped as zero", () => {
  const [, b] = align_series_for(
    [
      series("a", { x: "Q1", y: 1 }, { x: "Q2", y: 2 }),
      series("b", { x: "Q2", y: 3 }),
    ],
    "bar",
    UNSTACKED,
  );

  assert.deepEqual(b.data, [
    { x: "Q1", y: 0 },
    { x: "Q2", y: 3 },
  ]);
});

test("align_series_for leaves a treemap's regions their own labels", () => {
  const regions = [
    series(
      "North America",
      { x: "United States", y: 35 },
      { x: "Canada", y: 15 },
    ),
    series("Europe", { x: "France", y: 30 }, { x: "Germany", y: 55 }),
  ];

  assert.equal(align_series_for(regions, "treemap", UNSTACKED), regions);
});

test("align_series_for leaves a rangeBar timeline alone", () => {
  const tasks = [
    series("Design", { x: "Alice", y: [1, 5] }),
    series("Build", { x: "Bob", y: [4, 9] }),
  ];

  assert.equal(align_series_for(tasks, "rangeBar", UNSTACKED), tasks);
});

test("align_series_for leaves unstacked series without labels alone", () => {
  const lines = [
    series("a", { x: 1, y: 1 }, { x: 2, y: 2 }),
    series("b", { x: 2, y: 3 }),
  ];

  assert.equal(align_series_for(lines, "line", UNSTACKED), lines);
});
