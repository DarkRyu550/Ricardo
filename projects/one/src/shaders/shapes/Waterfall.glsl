#version 300 es
precision mediump float;

#define MAX_INSTANCES 2048
struct Instance
{
	/* Position offset in world space. */
	vec3 position;
	/* Scaling for each of the main vertices. */
	vec2 scaling;
};

layout(std140) uniform rc_global
{
	mat4 MountainWorldTransformation;
	mat4 SnowflakeWorldTransformation;
	mat4 BackwallWorldTransformation;
	mat4 WaterfallWorldTransformation;
	mat4 ViewProjectionTransformation;

	vec2  LightPosition;
	float FarPlane;
	vec3  LightColor;
	vec3  TransmissionTint;
};

layout(std140) uniform rc_mountains
{
	Instance Mountains[MAX_INSTANCES];
};

layout(std140) uniform rc_snowflakes
{
	Instance Snowflakes[MAX_INSTANCES];
};

layout(std140) uniform rc_backwalls
{
	Instance Backwalls[MAX_INSTANCES];
};

layout(std140) uniform rc_waterfalls
{
	Instance Waterfalls[MAX_INSTANCES];
};

in vec3 tt_vert_position;
in vec3 tt_vert_color;

out vec3  vs_color;
out vec3  vs_center_position;
out vec3  vs_vertex_position;
out vec2  vs_light_position;
out vec3  vs_light_color;
out float vs_far_plane;
out vec3  vs_tranmission_tint;

void main()
{
	vec4 position = vec4(tt_vert_position, 1.0);
	     position = WaterfallWorldTransformation * position;

	vec3 offset = Waterfalls[gl_InstanceID].position;
	position.x += offset.x;
	position.y += offset.y;
	position.z += offset.z;
	position = ViewProjectionTransformation * position;

	vec4 proj_offset = vec4(offset, 1.0);
	     proj_offset = ViewProjectionTransformation * proj_offset;

	vs_color            = tt_vert_color;
	vs_center_position  = (proj_offset / proj_offset.w).xyz;
	vs_vertex_position  = (position / position.w).xyz;
	vs_light_position   = LightPosition;
	vs_light_color      = LightColor;
	vs_far_plane        = FarPlane;
	vs_tranmission_tint = TransmissionTint;

	gl_Position = position;
}