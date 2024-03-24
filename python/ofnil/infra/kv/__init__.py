"""
This module provides interfaces and implementations of connectors for key-value stores
"""
from ofnil.infra.kv.feature_view import FeatureView
from ofnil.infra.kv.redis_feature_view import RedisFeatureView

interfaces = ["FeatureView"]
implementations = ["RedisFeatureView"]

__all__ = interfaces + implementations

FeatureView.subclasses = {"redis": RedisFeatureView}
