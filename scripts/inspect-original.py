"""Record source locations and numeric reference results without starting MoodTest.

Only the listed pure numeric/text/answer-key definitions are evaluated. The original's imports,
Pillow installer, GUI, network access and file export code are never executed.
"""
import __future__
import argparse
import ast
import hashlib
import json
import math
import operator
from pathlib import Path
import re
import xml.etree.ElementTree as ET
from decimal import Decimal, ROUND_HALF_UP
from types import SimpleNamespace
from typing import Dict, List, Optional, Tuple


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    args = parser.parse_args()
    raw = args.source.read_bytes()
    source = raw.decode("utf-8-sig")
    tree = ast.parse(source)
    lines = raw.splitlines(keepends=True)
    locations = []

    def visit(nodes, parent=""):
        for node in nodes:
            if isinstance(node, (ast.ClassDef, ast.FunctionDef)):
                name = f"{parent}.{node.name}" if parent else node.name
                if isinstance(node, ast.FunctionDef) and not parent.count("."):
                    offset = sum(map(len, lines[:node.lineno - 1])) + node.col_offset
                    locations.append({
                        "function": name, "line": node.lineno,
                        "end_line": node.end_lineno, "file_offset": hex(offset),
                        "native_address": None,
                        "calls": sorted({ast.unparse(n.func) for n in ast.walk(node)
                                         if isinstance(n, ast.Call)}),
                    })
                visit(node.body, name)

    visit(tree.body)
    functions = {
        "_eval_ast", "_preprocess_expr", "avaliar_expressao",
        "formatar_numero_pt", "remover_decimais_nulos", "_extract_variable_names",
        "avaliar_expressao_com_variaveis", "_replace_calculated_blocks",
        "_parse_correct_indices", "indices_para_letras", "letters_for",
    }
    assignments = {
        "_ALLOWED_OPS", "_ALLOWED_FUNCS", "_ALLOWED_CONSTS",
        "_DECIMAL_ZERO_PATTERN", "_PLACEHOLDER_PATTERN", "PLACEHOLDER_REGEX",
    }
    selected = [n for n in tree.body if
                isinstance(n, ast.FunctionDef) and n.name in functions or
                isinstance(n, ast.Assign) and any(
                    isinstance(t, ast.Name) and t.id in assignments for t in n.targets)]
    question = next(n for n in tree.body if isinstance(n, ast.ClassDef) and n.name == "Question")
    selected.append(next(n for n in question.body
                         if isinstance(n, ast.FunctionDef) and n.name == "_processar_texto"))
    namespace = dict(globals())
    exec(compile(ast.Module(body=selected, type_ignores=[]), str(args.source), "exec",
                 flags=__future__.annotations.compiler_flag), namespace)
    inputs = [
        ("Valor {= {a}^2/2} N", {"a": 4.0}),
        (r"  \(R = {r}\,\Omega\), \omega e \ohm  ", {"r": 1234.5}),
        ("Taxa 10,00; 1.000,000; 0,001; -2,00", {}),
        ("{={x}^2} / {=({x})^2} / {=x^2}", {"x": -2.0}),
        ("{=ln(1)+log10(100)+sqrt(4)+abs(-3)}", {}),
        ("{=2**3**2} / {=-2^2} / {=2^-2}", {}),
        ("{=1,005} / {=-1,005} / {=0,0049}", {}),
        ("{a} / {b} / {c}", {"a": 1.005, "b": -1.005, "c": 10.0}),
        ("{=sin(pi/2)+cos(0)+tan(0)+exp(0)+log(e)}", {}),
        ("{=2,675} / {=-2,675} / {=9,995} / {=-0,005}", {}),
        ("{=1e3+2e-2} / {=2^-3^2}", {}),
    ]
    cases = []
    for text, variables in inputs:
        values = {k: SimpleNamespace(value=v, decimals=2) for k, v in variables.items()}
        cases.append({"text": text, "variables": variables,
                      "expected": namespace["_processar_texto"](None, text, values)})
    path = Path(__file__).resolve().parents[1] / "mechanics.json"
    mechanics = json.loads(path.read_text(encoding="utf-8-sig"))
    mechanics["original"] = {
        "path": str(args.source.resolve()), "sha256": hashlib.sha256(raw).hexdigest(),
        "kind": "Python source", "native_addresses_available": False,
        "location_units": "zero-based byte offsets at def; source lines are one-based",
        "functions": locations,
        "reference_method": "isolated numeric/text AST definitions; original module not imported",
        "reference_cases": cases,
        "answer_score_cases": [
            {"fractions": scores, "expected": namespace["_parse_correct_indices"](
                [ET.Element("answer", fraction=score) for score in scores])}
            for scores in [[" 50 ", "0", "-25", " 0.5 ", "invalid"],
                           ["100", "100", "0"], ["", "NaN", "-1"]]
        ],
        "answer_letter_cases": [
            {"indices": indices, "expected": namespace["indices_para_letras"](indices)}
            for indices in [[], [0], [0, 2], [1, 3, 7]]
        ],
    }
    path.write_text(json.dumps(mechanics, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"Recorded {len(locations)} source functions and {len(cases)} reference cases in {path}")
    for case in cases:
        print(json.dumps(case, ensure_ascii=True))


if __name__ == "__main__":
    main()
