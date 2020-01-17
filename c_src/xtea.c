#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>

// Taken from Linux 5.3
// SPDX-License-Identifier: GPL-2.0-or-later
/* 
 * Cryptographic API.
 *
 * TEA, XTEA, and XETA crypto alogrithms
 *
 * The TEA and Xtended TEA algorithms were developed by David Wheeler 
 * and Roger Needham at the Computer Laboratory of Cambridge University.
 *
 * Due to the order of evaluation in XTEA many people have incorrectly
 * implemented it.  XETA (XTEA in the wrong order), exists for
 * compatibility with these implementations.
 *
 * Copyright (c) 2004 Aaron Grothe ajgrothe@yahoo.com
 */


#define XTEA_KEY_SIZE		16
#define XTEA_BLOCK_SIZE		8
#define XTEA_ROUNDS		32
#define XTEA_DELTA		0x9e3779b9

struct xtea_ctx {
  uint32_t KEY[4];
};

uint32_t le32_to_cpu(uint32_t x) {
#if __BIG_ENDIAN == 1
  return __bswap_32(x);
#else
  return x;
#endif
}

uint32_t cpu_to_le32(uint32_t x) {
  return le32_to_cpu(x);
}

static void xtea_encrypt(struct xtea_ctx *ctx, uint8_t *dst, const uint8_t *src)
{
  uint32_t y, z, sum = 0;
  uint32_t limit = XTEA_DELTA * XTEA_ROUNDS;
  const uint32_t *in = (const uint32_t *)src;
  uint32_t *out = (uint32_t *)dst;

  y = le32_to_cpu(in[0]);
  z = le32_to_cpu(in[1]);

  int i = 0;
  while (sum != limit) {
    /* printf("ROUND %08X %08X %08X\n",sum,y,z); */
    y += ((z << 4 ^ z >> 5) + z) ^ (sum + ctx->KEY[sum&3]); 
    sum += XTEA_DELTA;
    z += ((y << 4 ^ y >> 5) + y) ^ (sum + ctx->KEY[sum>>11 &3]); 
    i ++;
  }
	
  out[0] = cpu_to_le32(y);
  out[1] = cpu_to_le32(z);
}

int main(int argc, const char **argv) {
  uint32_t w[6];
  int i;
  struct xtea_ctx ctx;
  uint32_t x[2];
  uint32_t y[2];

  if (argc != 7) {
    fprintf(stderr,"usage: %s key1 key2 key3 key4 plain1 plain2 (6 32-bit hexadecimal words)\n",
	    argv[0]);
    exit(EXIT_FAILURE);
  }

  for (i = 0; i<6; i ++) {
    if (1 != sscanf(argv[1 + i],"%08x",w+i)) {
      fprintf(stderr,"%s: error in argument %d (%s)\n",argv[0],1+i,argv[1+i]);
      exit(EXIT_FAILURE);
    }
  }

  for (i = 0; i<4; i ++) {
    ctx.KEY[i] = w[i];
  }

  x[0] = w[4];
  x[1] = w[5];
  xtea_encrypt(&ctx, (uint8_t *) y, (const uint8_t *) x);

  printf("XTEA_[%08X %08X %08X %08X](%08X %08X)=%08X %08X\n",
	 ctx.KEY[0],
	 ctx.KEY[1],
	 ctx.KEY[2],
	 ctx.KEY[3],
	 x[0],
	 x[1],
	 y[0],
	 y[1]);

  return 0;
}

