import { expect, test, describe } from "bun:test";

describe("extra", () => {
  // Polytest Suite: extra

  describe("extra", () => {
    // Polytest Group: extra
    /* 
      Here
      Are
      New
      Lines
     */
    test.skip("desc_new_line", () => {
      throw Error('SHOULD BE SKIPPED!')
    });

  });

});
