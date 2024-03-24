ofnil.infra.graph
=========================

.. automodule:: ofnil.infra.graph
.. contents:: Contents
    :local:

.. currentmodule:: ofnil.infra.graph


Interfaces
-------------------------

.. autosummary::
   :nosignatures:

   {% for name in ofnil.infra.graph.interfaces %}
   {{ name }}
   {% endfor %}
 
{% for name in ofnil.infra.graph.interfaces %}
.. autoclass:: {{ name }}
   :members:
   :undoc-members:
   :show-inheritance:
{% endfor %}

Implementations
-------------------------

.. autosummary::
   :nosignatures:

   {% for name in ofnil.infra.graph.implementations %}
   {{ name }}
   {% endfor %}
 
{% for name in ofnil.infra.graph.implementations %}
.. autoclass:: {{ name }}
   :members:
   :undoc-members:
   :show-inheritance:
{% endfor %}