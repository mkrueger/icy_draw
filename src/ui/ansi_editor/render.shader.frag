#version 330

in vec2 UV;

uniform sampler2D u_render_texture;
uniform vec4      u_draw_rect;
uniform vec4      u_draw_area;
uniform float     u_effect;

out vec3 color;

void draw_background() {
	float checker_size = 8.0;
    vec2 p = floor(gl_FragCoord.xy / checker_size);
    float PatternMask = mod(p.x + mod(p.y, 2.0), 2.0);
	if (PatternMask < 1.0) {
		color = vec3(0.4, 0.4, 0.4);
	} else {
		color = vec3(0.6, 0.6, 0.6);
	}
}

void main() {
	vec2 uv   = (gl_FragCoord.xy - u_draw_rect.xy) / u_draw_rect.zw;
	vec2 from = u_draw_area.xy / u_draw_rect.zw;
	vec2 to   = u_draw_area.zw / u_draw_rect.zw;
	if (from.x <= uv.x && uv.x < to.x && 
	    from.y <= uv.y && uv.y < to.y) {
		color = texture(u_render_texture, (uv - from) / (to - from) ).xyz;
	} else {
		draw_background();
	}
}