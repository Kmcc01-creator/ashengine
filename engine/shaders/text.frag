#version 450

layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec4 fragColor;

layout(binding = 0) uniform sampler2D fontTexture;

layout(location = 0) out vec4 outColor;

void main() {
    float alpha = texture(fontTexture, fragTexCoord).r;
    outColor = vec4(fragColor.rgb, fragColor.a * alpha);
}
