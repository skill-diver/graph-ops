from typing import List

import redis

from ofnil.infra.kv.feature_view import FeatureView
from ofnil.ofnil import TableFeatureViewInfo


class RedisFeatureView(FeatureView):
    """Using columnar store key format: type/feature/id."""

    def __init__(self, feature_info: TableFeatureViewInfo):
        super().__init__(feature_info)
        uri = feature_info.infra_info["uri"]
        if uri.startswith("redis://"):
            uri = uri[8:]
        try:
            host, port = uri.split(":")
        except:
            raise ValueError(f'Unexpected uri {feature_info.infra_info["uri"]}')
        self.redis = redis.Redis(host=host, port=port)

    def _get_features(self, tlabel: str, feature_names: List[str], vids: list):
        if len(feature_names) == 0:
            return None
        if len(vids) == 0:
            return None
        return [
            val.decode("ascii") if val is not None else None
            for val in self.redis.mget(
                [f"{tlabel}/{feature_name}/{vid}" for feature_name in feature_names for vid in vids]
            )
        ]
