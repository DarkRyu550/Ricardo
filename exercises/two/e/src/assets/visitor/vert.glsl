#version 300 es
precision mediump float;

in vec3 tt_vert_position;
in vec2 tt_vert_texture;
in vec3 tt_vert_normal;
in vec3 tt_vert_tangent;
in vec3 tt_vert_bitangent;

out struct VS_OUT
{
	vec3 position;
	vec2 texture;

	mat3 ntb;
} vs_out;

layout(std140) uniform rc_params
{
	mat4 model_world_view;
};

void main()
{
	/* Culculate initial position of the dish from the model transformation. */
	vec4 dish = vec4(tt_vert_position, 1.0);
	dish = model_world_view * dish;

	/* Assemble the NTB matrix and the normal transformation matrix. */
	mat3 normal_transform = mat3(model_world_view);
	normal_transform = transpose(inverse(normal_transform));

	mat3 ntb;
	ntb[2] = tt_vert_normal;
	ntb[0] = tt_vert_tangent;
	ntb[1] = tt_vert_bitangent;

	ntb = normal_transform * ntb;

	/* Pass all needed information on to the next shader stage. */
	vs_out.ntb      = ntb;
	vs_out.texture  = vec2(tt_vert_texture.x, -tt_vert_texture.y);
	vs_out.position = (dish / dish.w).xyz;

	gl_Position = dish;
}
