#pragma once
#include <cstdint>

namespace {{namespace}} {
{% for st in structures %}
/// @brief {{st.abst}}
struct {{st.name}} {
  {%- for field in st.field %}
  /// @brief {{field.abst}}
  {{field.ctype}} {{field.name}}
  {%- if field.is_array() -%}
    {%- for dim in field.dims -%}
      [{{dim}}]
    {%- endfor -%}
  {%- endif -%};{% endfor %}
};
{% endfor %}
} // namespace {{namespace}}
