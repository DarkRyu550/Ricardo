#version 300 es
precision mediump float;

in vec3 tt_vert_position;
in vec3 tt_vert_normal;
in vec3 tt_vert_texture;

out vec3 frag_uv;
out vec3 frag_normal;

void main()
{
	frag_uv = tt_vert_texture;
	frag_normal = tt_vert_normal;

	gl_Position = vec4(tt_vert_position, 1.0);
}
