#define STB_IMAGE_IMPLEMENTATION
#include <iostream>
#include <chrono>
#include <filesystem>
#include "string.h"

#include "stb_image.h"

struct Histogram {
	unsigned int r[256];
	unsigned int g[256];
	unsigned int b[256];

	Histogram() : r{0}, g{0}, b{0} {}

	const std::string print() const {
		std::string s = "";
		for (int i = 0; i < 256; ++i)
			s += std::to_string(r[i]) + " ";
		s += "\n";
		for (int i = 0; i < 256; ++i)
			s += std::to_string(g[i]) + " ";
		s += "\n";
		for (int i = 0; i < 256; ++i)
			s += std::to_string(b[i]) + " ";
		s += "\n";

		return s;
	}
};

int main(){
	auto start = std::chrono::high_resolution_clock::now();
	Histogram hist;
	for (const auto & entry : std::filesystem::directory_iterator("../images")) {
		int width, height;
		int numberOfChannels;
		uint8_t *imageData = stbi_load(entry.path().c_str(), &width, &height, &numberOfChannels, 0);
		for (int i = 0; i < width * height * numberOfChannels; i += numberOfChannels) {
			uint8_t red = imageData[i];
			uint8_t green = imageData[i + 1];
			uint8_t blue = imageData[i + 2];
			hist.r[red]++;
			hist.g[green]++;
			hist.b[blue]++;
		}
	}
	auto end = std::chrono::high_resolution_clock::now();

    std::chrono::duration<double, std::milli> ms = end - start;

	std::cout << hist.print() << "time: " << ms.count() << "ms" << std::endl;
	return 0;
}
