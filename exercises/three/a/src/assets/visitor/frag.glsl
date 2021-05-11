#version 300 es
precision mediump float;

in struct VS_OUT
{
	vec3 position;
	vec2 texture;

	mat3 ntb;
} vs_out;

out vec4 color;

void main()
{
	color = vec4(vs_out.ntb[2], 1.0);
}
