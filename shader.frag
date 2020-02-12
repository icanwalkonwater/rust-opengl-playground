#version 330 core

in vec3 vertPos;

out vec4 FragColor;

uniform vec4 albedo;

void main() {
    FragColor = vec4(vertPos / 2 + .5, 1);
    // FragColor = albedo;
}
