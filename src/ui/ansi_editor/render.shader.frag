#version 330
in vec2 UV;

uniform sampler2D u_render_texture;
uniform vec4      u_draw_rect;
uniform vec4      u_draw_area;
uniform float     u_effect;

out vec3 color;

void main() {
	/*
	vec2 uv   = (gl_FragCoord.xy - u_draw_rect.xy) / u_draw_rect.zw;

	vec2 from = u_draw_area.xy / u_draw_rect.zw;
	vec2 to   = u_draw_area.zw / u_draw_rect.zw;

	if (from.x <= uv.x && uv.x < to.x && 
	    from.y <= uv.y && uv.y < to.y) {
		color = texture(u_render_texture, (uv - from) / (to - from) ).xyz;
	} else {
		color = vec3(1.0);
	}*/

	color = vec3(1.0);
}