varying vec2 v_vt;
varying vec2 v_pos;
varying vec4 v_color;

#ifdef VERTEX_SHADER
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform mat3 u_model_matrix;

attribute vec2 a_pos;
attribute vec2 a_vt;
attribute vec4 a_color;

void main() {
    v_vt = a_vt;
    v_color = a_color;
    v_pos = a_pos;
    vec3 pos = u_projection_matrix * u_view_matrix * u_model_matrix * vec3(a_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform float u_cut_x;
uniform sampler2D u_texture_left;
uniform sampler2D u_texture_right;

void main() {
    vec4 in_color;
    if (v_pos.x <= u_cut_x) {
        in_color = texture2D(u_texture_left, v_vt);
    } else {
        in_color = texture2D(u_texture_right, v_vt);
    }

    gl_FragColor = in_color * u_color * v_color;
}
#endif
