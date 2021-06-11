#version 300 es
precision mediump float;

in vec3  vs_color;
in vec3  vs_center_position;
in vec3  vs_vertex_position;
in vec2  vs_light_position;
in vec3  vs_light_color;
in float vs_far_plane;
in vec3  vs_tranmission_tint;

out vec4 color;
void main()
{
	vec2  light_direction = normalize(vs_light_position - vs_center_position.xy);
	float alignment    = dot(vs_vertex_position.xy - vs_center_position.xy, light_direction);
	float transmission = clamp(vs_vertex_position.z, 0.0, 1.0);

	vec3 tint = vs_light_color * vs_tranmission_tint;
	vec3 albd = vs_light_color * vs_color;
	if(alignment < 0.0)
		albd *= 0.5;

	color = vec4(mix(albd, tint, transmission), 1.0);
}
