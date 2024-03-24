from abc import ABC, abstractmethod
from typing import List

from ofnil.ofnil import FeatureServingOutputType, TableFeatureViewInfo


class FeatureView(ABC):
    subclasses = {}  # init from the kv module __init__ file

    def __new__(cls, feature_info: TableFeatureViewInfo, *args, **kwargs):
        """Factory method for FeatureView

        Parameters
        ----------
        feature_info : TableFeatureViewInfo

        Returns
        -------
        A subclass of FeatureView

        Raises
        ------
        ValueError
            Currently the supported infra is redis. The error is raised if feature_info.infra_info["infra_type"] has other values.
        """
        if cls is FeatureView:
            infra_type = feature_info.infra_info["infra_type"]
            if infra_type in FeatureView.subclasses:
                return super().__new__(FeatureView.subclasses[infra_type])
            else:
                raise ValueError(
                    f"Table feature view resides in infra {infra_type} which does not support feature retrieval. Support list {FeatureView.subclasses.keys()}"
                )
        else:
            return super().__new__(cls)

    def __init__(self, feature_info: TableFeatureViewInfo) -> None:
        self.feature_info = feature_info

    def get(self, ids: list):
        """Get all features of the given entities

        Parameters
        ----------
        ids : list
            A list of entity ids

        Returns
        -------
        A tuple of feature vectors and subgraph
            The desired model input according to the rendering opts.
        """
        if self.feature_info.rendering_opt.output_type == FeatureServingOutputType.NdArray:
            import numpy as np

            return np.array(
                self._get_features(self.feature_info.entity_label, self.feature_info.field_names, ids)
            ).reshape((-1, len(self.feature_info.field_names)))
        return self._get_features(self.feature_info.entity_label, self.feature_info.field_names, ids)

    @abstractmethod
    def _get_features(self, tlabel: str, feature_names: List[str], ids: list):
        # TODO(tatiana): doc
        pass
