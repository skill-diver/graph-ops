from ctypes import ArgumentError
from typing import Any, Dict

from ..ofnil import Graph
from .procedure import Procedure


class BuiltIn(Procedure):
    def __init__(self, procedure, args: Dict[str, Any] = None, name=None):
        super().__init__()
        self.procedure_name = procedure
        self.name = procedure if name is None else name
        self.args = args

    def update_procedure_args(self, args: Dict[str, Dict[str, Any]]):
        if self.name in args:
            self.args.update(args[self.name])

    def construct(self, input):
        if isinstance(input, Graph):
            input = input.transform(self.context)
        return input.apply_procedure(
            self.context,
            self.procedure_name,
            self.args,
        )

    def __str__(self) -> str:
        return f"{super().__str__()} {self.name}: {self.procedure_name}({self.args})"
