#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

IMPLEMENTATIONS = ("hex", "muhex", "faster-hex")
OPERATIONS = ("encode", "decode")


@dataclass(frozen=True)
class Measurement:
    operation: str
    implementation: str
    size_label: str
    size_bytes: int
    throughput_bps: float


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate BENCHMARKS.md from Criterion benchmark output."
    )
    parser.add_argument(
        "--criterion-root",
        default="target/criterion",
        type=Path,
        help="Path to Criterion output root.",
    )
    parser.add_argument(
        "--group",
        default="encdec",
        help="Criterion benchmark group to parse.",
    )
    parser.add_argument(
        "--operation",
        choices=("all", "encode", "decode"),
        default="all",
        help="Operation table(s) to include.",
    )
    parser.add_argument(
        "--output",
        default=Path("BENCHMARKS.md"),
        type=Path,
        help="Output markdown file.",
    )
    return parser.parse_args()


def parse_measurements(group_dir: Path) -> dict[tuple[str, str, str], Measurement]:
    if not group_dir.exists():
        raise FileNotFoundError(f"Criterion group not found: {group_dir}")

    measurements: dict[tuple[str, str, str], Measurement] = {}

    for bench_path in sorted(group_dir.rglob("new/benchmark.json")):
        with bench_path.open("r", encoding="utf-8") as f:
            bench = json.load(f)

        function_id = bench.get("function_id", "")
        if "/" not in function_id:
            continue

        operation, implementation = function_id.split("/", 1)
        if operation not in OPERATIONS or implementation not in IMPLEMENTATIONS:
            continue

        size_label = bench.get("value_str")
        size_bytes = bench.get("throughput", {}).get("Bytes")
        if not isinstance(size_label, str) or not isinstance(size_bytes, int):
            continue

        estimates_path = bench_path.with_name("estimates.json")
        if not estimates_path.exists():
            continue

        with estimates_path.open("r", encoding="utf-8") as f:
            estimates = json.load(f)
        mean_ns = estimates.get("mean", {}).get("point_estimate")
        if not isinstance(mean_ns, (int, float)) or mean_ns <= 0:
            continue

        throughput_bps = (size_bytes * 1e9) / float(mean_ns)
        key = (operation, size_label, implementation)
        measurements[key] = Measurement(
            operation=operation,
            implementation=implementation,
            size_label=size_label,
            size_bytes=size_bytes,
            throughput_bps=throughput_bps,
        )

    return measurements


def format_throughput(throughput_bps: float) -> str:
    throughput_mib = throughput_bps / (1024 * 1024)
    if throughput_mib >= 1024:
        return f"{throughput_mib / 1024:.2f} GiB/s"
    return f"{throughput_mib:.2f} MiB/s"


def format_speedup(speedup: float | None) -> str:
    if speedup is None:
        return "-"
    return f"{speedup:.2f}x"


def is_fastest(value: float, fastest: float) -> bool:
    tolerance = max(abs(fastest), 1.0) * 1e-12
    return abs(value - fastest) <= tolerance


def render_operation_table(
    operation: str, measurements: dict[tuple[str, str, str], Measurement]
) -> list[str]:
    rows: dict[str, dict[str, float]] = {}
    size_bytes: dict[str, int] = {}

    for (op, size_label, implementation), measurement in measurements.items():
        if op != operation:
            continue
        rows.setdefault(size_label, {})[implementation] = measurement.throughput_bps
        size_bytes[size_label] = measurement.size_bytes

    lines: list[str] = [f"## {operation.capitalize()} Throughput", ""]
    if not rows:
        lines.extend(["_No results found._", ""])
        return lines

    lines.extend(
        [
            "| size | hex | muhex | faster-hex | muhex vs hex | muhex vs faster-hex |",
            "| --- | ---: | ---: | ---: | ---: | ---: |",
        ]
    )

    for size_label in sorted(rows, key=lambda size: (-size_bytes[size], size)):
        values = rows[size_label]
        available = [v for v in values.values()]
        fastest = max(available) if available else None

        rendered: dict[str, str] = {}
        for implementation in IMPLEMENTATIONS:
            value = values.get(implementation)
            if value is None:
                rendered[implementation] = "-"
                continue

            formatted = format_throughput(value)
            if fastest is not None and is_fastest(value, fastest):
                formatted = f"**{formatted}**"
            rendered[implementation] = formatted

        hex_value = values.get("hex")
        muhex_value = values.get("muhex")
        faster_hex_value = values.get("faster-hex")
        muhex_vs_hex = None
        if muhex_value is not None and hex_value is not None:
            muhex_vs_hex = muhex_value / hex_value
        muhex_vs_faster_hex = None
        if muhex_value is not None and faster_hex_value is not None:
            muhex_vs_faster_hex = muhex_value / faster_hex_value

        lines.append(
            f"| `{size_label}` | {rendered['hex']} | {rendered['muhex']} | "
            f"{rendered['faster-hex']} | {format_speedup(muhex_vs_hex)} | "
            f"{format_speedup(muhex_vs_faster_hex)} |"
        )

    lines.append("")
    return lines


def render_markdown(
    criterion_root: Path,
    group: str,
    operation: str,
    measurements: dict[tuple[str, str, str], Measurement],
) -> str:
    lines = [
        "# Benchmarks",
        "",
        f"Generated from Criterion output in `{criterion_root / group}`.",
        "Throughput is computed from `throughput.Bytes` and `new/estimates.json` mean time.",
        "",
    ]

    selected_operations = [operation] if operation != "all" else list(OPERATIONS)
    for op in selected_operations:
        lines.extend(render_operation_table(op, measurements))

    return "\n".join(lines)


def main() -> int:
    args = parse_args()
    group_dir = args.criterion_root / args.group

    try:
        measurements = parse_measurements(group_dir)
    except FileNotFoundError as err:
        print(err, file=sys.stderr)
        return 1

    if not measurements:
        print(f"No benchmark measurements found under: {group_dir}", file=sys.stderr)
        return 1

    markdown = render_markdown(
        criterion_root=args.criterion_root,
        group=args.group,
        operation=args.operation,
        measurements=measurements,
    )
    args.output.write_text(markdown, encoding="utf-8")
    print(f"Wrote {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
