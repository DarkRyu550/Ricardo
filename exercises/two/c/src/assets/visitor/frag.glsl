#version 300 es
precision mediump float;

in struct VS_OUT
{
	vec3 color;
} vs_out;

out vec4 color;
void main()
{
	color = vec4(vs_out.color, 1.0);
}
