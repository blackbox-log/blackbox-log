#include <stdint.h>
#include "../upstream/src/stream.h"
#include "../upstream/src/tools.h"

int32_t streamReadNeg14Bit(mmapStream_t* stream) {
	return -signExtend14Bit(streamReadUnsignedVB(stream));
}
