'use strict';

export default {
  formatPercentage(value) {
    if(value >= 1.0)
      return '100%';
    if(value <= 0.0)
      return '0%';
    if(value >= 0.1)
      return parseFloat(value * 100).toPrecision(2) + '%';
    return parseFloat(value * 100).toPrecision(1) + '%';
  },
};
