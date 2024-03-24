ofnil
=============

.. currentmodule:: ofnil

Obtaining an Ofnil Client
----------------------------------------

Ofnil offers a :class:`Client` class as an endpoint for feature registry, feature deployment, and feature serving.

.. automodule:: ofnil.client
   :members:
   :undoc-members:
   :show-inheritance:

Registering a Source Graph Manually
----------------------------------------

.. currentmodule:: ofnil

.. autosummary::
   :toctree: ../generated/
   :recursive:

   vertex_entity
   edge_entity
   fields
   Graph

Users can register a source graph in feature registry by manual specification:

1. Define entities in the source graph by :func:`vertex_entity` and :func:`edge_entity`
2. Define fields (properties) of an entity in the source graph by :func:`fields`
3. Define the source graph :class:`Graph`
