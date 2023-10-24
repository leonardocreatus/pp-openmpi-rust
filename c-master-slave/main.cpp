#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"
#include <iostream>

int main(){
    int width, height; // width and height of the picture
    int numberOfChannels; // something like 1 for grayscale image, 3 for rgb, 4 for rgba...
    uint8_t *imageData = stbi_load("a-bug-in-louisiana.jpg", &width, &height, &numberOfChannels, 0);
    
    for (int i = 0; i < width * height * numberOfChannels; i += numberOfChannels) {
        uint8_t red = imageData[i];
        uint8_t green = imageData[i + 1];
        uint8_t blue = imageData[i + 2];
        std::cout << "Pixel " << i / numberOfChannels << " has color: " << (int)red << ", " << (int)green << ", " << (int)blue << std::endl;
    }
}