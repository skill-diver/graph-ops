from typing import Iterator, OrderedDict, overload

from .procedure import Procedure


class Sequential(Procedure):
    @overload
    def __init__(self, *args: Procedure):
        ...

    @overload
    def __init__(self, arg: OrderedDict[str, Procedure]):
        ...

    def __init__(self, *args):
        super().__init__()
        if len(args) == 1 and isinstance(args[0], OrderedDict):
            for key, procedure in args[0].items():
                self.add_procedure(key, procedure)
        for idx, procedure in enumerate(args):
            self.add_procedure(str(idx), procedure)

    def __iter__(self) -> Iterator[Procedure]:
        return iter(self._procedures.values())

    def construct(self, input):
        for procedure in self:
            print(procedure)
            input = procedure(input)
        return input
