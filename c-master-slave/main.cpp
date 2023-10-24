#define STB_IMAGE_IMPLEMENTATION
#define STB_IMAGE_RESIZE_IMPLEMENTATION
#include <iostream>
#include <chrono>
#include <filesystem>
#include "string.h"

#include "stb_image.h"
#include "stb_image_resize2.h"

#include "mpi.h"

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

	void join(Histogram const& other) {
		for (int i = 0; i < 256; ++i) {
			this->r[i] += other.r[i];
			this->g[i] += other.g[i];
			this->b[i] += other.b[i];
		}
	}
};

int main(int argc, char* argv[]){
	auto start = std::chrono::high_resolution_clock::now();

	MPI_Init(&argc, &argv);
	int rank, size;
	MPI_Comm_size(MPI_COMM_WORLD, &size);
	MPI_Comm_rank(MPI_COMM_WORLD, &rank);

	if (size < 2) {
		std::cerr << "Error: must have at least 2 processes!" << std::endl;
		MPI_Finalize();
		return 1;
	}

	if (rank == 0) {
		// Mestre
		int send_to = 1;
		Histogram hist;
		Histogram received;
		MPI_Status status;
		for (const auto & entry : std::filesystem::directory_iterator("../images")) {
			const std::string filename = entry.path().string();
			fflush(stdout);
			MPI_Send(filename.c_str(), filename.size(), MPI_CHAR, send_to, 0, MPI_COMM_WORLD);

			++send_to;
			if (send_to == size) {
				send_to = 1;
			}
		}

		for (int i = 1; i < size; ++i) {
			MPI_Send("\0", 1, MPI_CHAR, i, 0, MPI_COMM_WORLD);
		}

		for (int i = 1; i < size; ++i) {
			MPI_Recv(&received, 256 * 3, MPI_UNSIGNED, MPI_ANY_SOURCE, MPI_ANY_TAG, MPI_COMM_WORLD, &status);
			hist.join(received);
		}

		auto end = std::chrono::high_resolution_clock::now();
		std::chrono::duration<double, std::milli> ms = end - start;
		std::cout << hist.print() << "time: " << ms.count() << "ms" << std::endl;
	} else {
		// Trabalhador
		int width, height;
		int numberOfChannels;
		MPI_Status status;
		Histogram hist;
		while (true) {
			char buf[256] = {0};
			MPI_Recv(&buf, 256, MPI_CHAR, 0, 0, MPI_COMM_WORLD, &status);

			if (buf[0] == '\0') {
				break;
			}

			uint8_t *imageData = stbi_load(buf, &width, &height, &numberOfChannels, 0);
			uint8_t *resized =  stbir_resize_uint8_srgb(
					imageData,
					width,
					height,
					numberOfChannels,
					0,
					width * 32,
					height * 32,
					0,
					STBIR_RGBA);
			for (int i = 0; i < width * height * numberOfChannels; i += numberOfChannels) {
				uint8_t red = resized[i];
				uint8_t green = resized[i + 1];
				uint8_t blue = resized[i + 2];
				hist.r[red]++;
				hist.g[green]++;
				hist.b[blue]++;
			}
			std::free(imageData);
			std::free(resized);

		}
		MPI_Send(&hist, 256 * 3, MPI_UNSIGNED, 0, 0, MPI_COMM_WORLD);
	}

	MPI_Finalize();
	return 0;
}
