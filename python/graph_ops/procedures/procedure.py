from typing import Optional, OrderedDict, Tuple

from ofnil.ofnil import PipelineContext


class Procedure:
    def __init__(self, context: PipelineContext = None):
        self._procedures = OrderedDict()
        self.context = context

    def apply(self, fn):
        for proc in self.children():
            proc.apply(fn)
        fn(self)
        return self

    def apply_context(self, context):
        self.apply(lambda proc: proc.set_context(context))

    def set_context(self, context):
        self.context = context

    def add_procedure(self, name: str, proc: Optional["Procedure"]) -> None:
        if not isinstance(proc, Procedure):
            raise TypeError(f"{type(proc)} is not a Procedure subclass")
        elif not isinstance(name, str):
            raise TypeError(f"Procedure name should be a string. Got {type(name)}")
        elif hasattr(self, name) and name not in self._procedures:
            raise KeyError("Attribute '{}' already exists".format(name))
        elif name == "":
            raise KeyError("Procedure name can't be empty string")
        self._procedures[name] = proc
        if self.context is not None:
            proc.set_context(self.context)

    def children(self):
        for _, procedure in self._procedures.items():
            yield procedure

    def _call_impl(self, *args, **kwargs):
        if not hasattr(self, "context"):
            raise RuntimeError(f"Pipeline context is not initialized. Call {self}.apply_context()?")
        return self.construct(*args, **kwargs)

    __call__ = _call_impl

    def construct(self, *args, **kwargs):
        return [proc(*args, **kwargs) for proc in self.children()]

    def finalize(self, entities=None, fields=None, topos=None):
        fields = list(self._export_dataframe_fields(fields))
        return self.context.finalize(entities=entities, fields=fields, topos=topos)

    def _export_dataframe_fields(self, fields):
        for item in fields:
            if isinstance(item, Tuple):
                dataframes, infra = item
                if not isinstance(dataframes, list):
                    for field in dataframes.export(infra):
                        yield field
                for dataframe in dataframes:
                    for field in dataframe.export(infra):
                        yield field
            else:
                yield item
