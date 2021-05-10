#version 300 es
precision mediump float;

uniform sampler2D tt_tex_albedo;
uniform sampler2D tt_tex_normal;
uniform sampler2D tt_tex_roughness;
uniform sampler2D tt_tex_metallic;

in struct VS_OUT
{
	vec3 position;
	vec2 texture;

	mat3 ntb;
} vs_out;

out vec4 color;

void main()
{
	color = vec4(texture(tt_tex_albedo, vs_out.texture).xyz, 1.0);
}
