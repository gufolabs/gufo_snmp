# ---------------------------------------------------------------------
# Gufo SNMP: Generate benchmark charts
# ---------------------------------------------------------------------
# Copyright (C) 2024-25, Gufo Labs
# See LICENSE.md for details
# ---------------------------------------------------------------------
"""Parse bechmark results and generate charts."""

# Python modules
import re
from dataclasses import dataclass

# Third-party modules
import matplotlib.pyplot as plt
from matplotlib import ticker

rx_name = re.compile(r"^Name \(time in (\S+)\)")


@dataclass
class BenchmarkItem(object):
    """
    Single benchmark item.

    Attributes:
        name: Benchmark name.
        value: Benchmark value.
        measure: Units measure.
    """

    name: str
    value: float
    measure: str

    @property
    def is_ours(self) -> bool:
        """Check if it is a benchmark of our product."""
        return "Gufo" in self.name

    @property
    def is_async(self) -> bool:
        """Check if it is an async version."""
        return "(Async)" in self.name


@dataclass
class Benchmark(object):
    """Benchmark descriptor.

    Attributes:
        path: Output chart path.
        title: Chart title.
        data: Data file path
    """

    path: str
    title: str
    data: str

    def get_results(self) -> list[BenchmarkItem]:
        """
        Get benchmark results.

        Returns:
            List of collected benchmark items
        """
        r: list[BenchmarkItem] = []
        with open(self.data) as fp:
            measure: str | None = None
            wait_for_start = True
            for line in fp:
                line = line.strip()
                if not line:
                    continue
                if not measure:
                    match = rx_name.search(line)
                    if match:
                        measure = match.group(1)
                elif wait_for_start:
                    if line.startswith("---"):
                        wait_for_start = False
                elif line.startswith("---"):
                    break
                else:
                    parts = line.split()
                    name = self.normalize_name(parts[0])
                    value = float(parts[9].replace(",", ""))
                    r.append(
                        BenchmarkItem(name=name, value=value, measure=measure)
                    )
        return r

    @staticmethod
    def normalize_name(s: str) -> str:
        """Normalize test name.

        Args:
            s: Test name.

        Returns:
            Normalized name.
        """
        if s.startswith("test_"):
            s = s[5:]
        mode = ""
        if s.endswith("_sync"):
            s = s[:-5]
            mode = " (Sync)"
        elif s.endswith("_async"):
            s = s[:-6]
            mode = " (Async)"
        s = NAME_MAP.get(s, s)
        return f"{s}{mode}"

    @property
    def test_name(self) -> str:
        """Literal test name."""
        parts = self.data.split("/")
        version = parts[2]
        tn = self.data[:-4].split("_")[-1]
        name = f"SNMP{version} {tn.upper()}"
        if "p4" in self.data:
            name = f"{name} (x4)"
        return name


BENCHMARKS = [
    Benchmark(
        title="SNMP v2c GETNEXT (Median)",
        path="docs/benchmarks/v2c/getnext.png",
        data="docs/benchmarks/v2c/test_v2c_getnext.txt",
    ),
    Benchmark(
        title="SNMP v2c GETBULK (Median)",
        path="docs/benchmarks/v2c/getbulk.png",
        data="docs/benchmarks/v2c/test_v2c_getbulk.txt",
    ),
    Benchmark(
        title="4 parallel SNMP v2c GETNEXT (Median)",
        path="docs/benchmarks/v2c/getnext_p.png",
        data="docs/benchmarks/v2c/test_v2c_p4_getnext.txt",
    ),
    Benchmark(
        title="4 parallel SNMP v2c GETBULK (Median)",
        path="docs/benchmarks/v2c/getbulk_p.png",
        data="docs/benchmarks/v2c/test_v2c_p4_getbulk.txt",
    ),
    Benchmark(
        title="SNMP v3 GETNEXT (Median)",
        path="docs/benchmarks/v3/getnext.png",
        data="docs/benchmarks/v3/test_v3_getnext.txt",
    ),
    Benchmark(
        title="SNMP v3 GETBULK (Median)",
        path="docs/benchmarks/v3/getbulk.png",
        data="docs/benchmarks/v3/test_v3_getbulk.txt",
    ),
    Benchmark(
        title="4 parallel SNMP v3 GETNEXT (Median)",
        path="docs/benchmarks/v3/getnext_p.png",
        data="docs/benchmarks/v3/test_v3_p4_getnext.txt",
    ),
    Benchmark(
        title="4 parallel SNMP v3 GETBULK (Median)",
        path="docs/benchmarks/v3/getbulk_p.png",
        data="docs/benchmarks/v3/test_v3_p4_getbulk.txt",
    ),
]

NAME_MAP = {"gufo_snmp": "Gufo SNMP"}


def build_barchart(bench: Benchmark, data: list[BenchmarkItem]) -> None:
    """
    Build bar chart into PNG file.

    Args:
        bench: Benchmark description.
        data: List of benchmark items.
    """

    def is_gufo_snmp(s: str) -> bool:
        return "Gufo SNMP" in s

    # Extracting test names and measured values from the data
    tests = [x.name for x in data]
    values = [x.value for x in data]
    scale = data[0].measure

    # Creating the bar chart
    plt.figure(figsize=(10, 6))
    plt.barh(
        tests,
        values,
        color=[
            "#2c3e50" if is_gufo_snmp(test) else "#34495e" for test in tests
        ],
    )
    plt.xlabel(f"Time ({scale})")
    plt.title(bench.title)
    # Adding thousands separator to y-axis labels
    plt.gca().xaxis.set_major_formatter(ticker.StrMethodFormatter("{x:,.0f}"))
    # Adding text annotations for ratio between each bar and smallest one
    min_value = min(values)
    for test, value in zip(tests, values):
        ratio = value / min_value
        fontweight = "bold" if is_gufo_snmp(test) else "normal"
        plt.text(
            value, test, f" x{ratio:.2f}", va="center", fontweight=fontweight
        )
    # Make y-axis labels bold for test names containing "gufo_http"
    for tick_label in plt.gca().get_yticklabels():
        if is_gufo_snmp(tick_label.get_text()):
            tick_label.set_weight("bold")
    # Adjusting right padding to shift border to the right
    plt.subplots_adjust(right=1.3)
    # Saving the plot as an SVG file
    print(f"Writing {bench.path}")
    plt.savefig(bench.path, format="png", bbox_inches="tight")
    plt.close()


def write_summary(scale: str, data: dict[str, list[float]]) -> None:
    """Write summary table."""
    path = "docs/benchmarks/conclusions.txt"
    print(f"Writing {path}")
    r = [
        f"| Test | Sync ({scale}) | Async ({scale}) | Async<br>overhead |",
        "| --- | ---: | ---: | ---: |",
    ]
    for tn, tv in data.items():
        overhead = (tv[1] - tv[0]) * 100.0 / tv[1]
        r.append(f"| {tn} | {tv[0]:.2f} | {tv[1]:.2f} | {overhead:.2f}% |")
    r.append("")
    with open(path, "w") as fp:
        fp.write("\n".join(r))


def main() -> None:
    """Main function."""
    summary: dict[str, list[float]] = {}  # test -> [sync, async]
    for bench in BENCHMARKS:
        results = bench.get_results()
        build_barchart(bench, results)
        # Contribute to summary table
        scale = results[0].measure
        summary[bench.test_name] = [0.0, 0.0]
        for item in results:
            if not item.is_ours:
                continue
            i = 1 if item.is_async else 0
            summary[bench.test_name][i] = item.value
    write_summary(scale, summary)


if __name__ == "__main__":
    main()
