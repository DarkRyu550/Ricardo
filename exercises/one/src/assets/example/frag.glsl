#version 300 es
precision mediump float;

uniform sampler2D tt_texture;

in vec3 frag_uv;
in vec3 frag_normal;

out vec4 color;

void main()
{
	color = vec4(frag_uv, 1.0);
}
