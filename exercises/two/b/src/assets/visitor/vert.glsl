#version 300 es
precision mediump float;

in vec3 tt_vert_position;
in vec3 tt_vert_texture;
in vec3 tt_vert_normal;
in vec3 tt_vert_tangent;
in vec3 tt_vert_bitangent;

out struct VS_OUT
{
	vec3 color;
} vs_out;

layout(std140) uniform rc_params
{
	mat4 model_world_view;
};

void main()
{
	vs_out.color = tt_vert_normal;

	vec4 position = vec4(tt_vert_position, 1.0);
	gl_Position = model_world_view * position;
}
