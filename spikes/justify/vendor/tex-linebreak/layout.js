/* tex-linebreak 0.9.0 src/layout.ts. SPDX license identifier MIT. See LICENSE.
 * Layout core only. The HTML writer from html.ts is not in this file.
 */
function createTexLayout() {
  var MIN_COST = -1000;
  var MAX_COST = 1000;
  var MIN_ADJUSTMENT_RATIO = -1;
  var PARAGRAPH_START = -1;
  function isForcedBreak(item) {
    return item.type === "penalty" && item.cost <= MIN_COST;
  }
  function lineStartsWithContent(item) {
    return item.type === "box" || item.type === "penalty" && item.cost >= MAX_COST;
  }
  function lineStartIndex(items, breakpoint) {
    let start;
    if (breakpoint === PARAGRAPH_START) {
      start = 0;
    } else {
      start = breakpoint + (items[breakpoint].type === "penalty" ? 1 : 0);
    }
    while (start < items.length && !lineStartsWithContent(items[start])) {
      start++;
    }
    return start;
  }
  function lineContentStart(items, startBreakpoint, endBreakpoint) {
    return Math.min(lineStartIndex(items, startBreakpoint), endBreakpoint + 1);
  }
  var defaultOptions = {
    maxAdjustmentRatio: null,
    initialMaxAdjustmentRatio: 1,
    doubleHyphenPenalty: 0,
    adjacentLooseTightPenalty: 0
  };
  var MaxAdjustmentExceededError = class extends Error {
  };
  function breakLines(items, lineLengths, opts = {}) {
    if (items.length === 0) {
      return [];
    }
    if (!isForcedBreak(items[items.length - 1])) {
      throw new Error("Paragraph items must end with a forced break");
    }
    const hasNegativeStretchOrShrink = items.some(
      (item) => item.type === "glue" && (item.stretch < 0 || item.shrink < 0)
    );
    const hasNegativeWidths = items.some((item) => item.width < 0);
    const prefixSums = items.reduce(
      (sums, item, i) => {
        sums.boxWidth[i + 1] = sums.boxWidth[i] + (item.type === "box" ? item.width : 0);
        sums.glueWidth[i + 1] = sums.glueWidth[i] + (item.type === "glue" ? item.width : 0);
        sums.glueStretch[i + 1] = sums.glueStretch[i] + (item.type === "glue" ? item.stretch : 0);
        sums.glueShrink[i + 1] = sums.glueShrink[i] + (item.type === "glue" ? item.shrink : 0);
        return sums;
      },
      {
        boxWidth: new Array(items.length + 1).fill(0),
        glueWidth: new Array(items.length + 1).fill(0),
        glueStretch: new Array(items.length + 1).fill(0),
        glueShrink: new Array(items.length + 1).fill(0)
      }
    );
    const suffixMinBreakpointFloor = new Array(items.length).fill(Infinity);
    const isFeasibleBreakpoint = (b) => {
      const item = items[b];
      return item.type === "glue" && b > 0 && items[b - 1].type === "box" || item.type === "penalty" && item.cost < MAX_COST;
    };
    if (!hasNegativeStretchOrShrink) {
      for (let b = 0; b < items.length; b++) {
        if (isFeasibleBreakpoint(b)) {
          const item = items[b];
          const penaltyWidth = item.type === "penalty" ? item.width : 0;
          suffixMinBreakpointFloor[b] = prefixSums.boxWidth[b + 1] + prefixSums.glueWidth[b] - prefixSums.glueShrink[b] + penaltyWidth;
        }
      }
      let minFloor = Infinity;
      for (let i = items.length - 1; i >= 0; i--) {
        const floor = suffixMinBreakpointFloor[i];
        if (floor !== Infinity) {
          suffixMinBreakpointFloor[i] = minFloor;
          minFloor = Math.min(minFloor, floor);
        }
      }
    }
    const opts_ = { ...defaultOptions, ...opts };
    const lineLen = (i) => Array.isArray(lineLengths) ? lineLengths[i] : lineLengths;
    const currentMaxAdjustmentRatio = Math.min(
      opts_.initialMaxAdjustmentRatio,
      opts_.maxAdjustmentRatio !== null ? opts_.maxAdjustmentRatio : Infinity
    );
    const active = /* @__PURE__ */ new Set();
    const nextContent = new Array(items.length + 1);
    nextContent[items.length] = items.length;
    for (let i = items.length - 1; i >= 0; i--) {
      nextContent[i] = lineStartsWithContent(items[i]) ? i : nextContent[i + 1];
    }
    active.add({
      index: PARAGRAPH_START,
      line: 0,
      // Fitness is ignored for this node.
      fitness: 0,
      isFallback: false,
      totalDemerits: 0,
      prev: null
    });
    const lineMetrics = (startBreakpoint, endBreakpoint) => {
      const rawStart = startBreakpoint === PARAGRAPH_START ? nextContent[0] : nextContent[startBreakpoint + (items[startBreakpoint].type === "penalty" ? 1 : 0)];
      const start = Math.min(rawStart, endBreakpoint + 1);
      if (start > endBreakpoint) {
        return { actualLen: 0, lineStretch: 0, lineShrink: 0, startOffset: 0 };
      }
      const interiorGlueStart = Math.min(start + 1, endBreakpoint);
      const penaltyWidth = items[endBreakpoint].type === "penalty" ? items[endBreakpoint].width : 0;
      return {
        actualLen: prefixSums.boxWidth[endBreakpoint + 1] - prefixSums.boxWidth[start] + (prefixSums.glueWidth[endBreakpoint] - prefixSums.glueWidth[interiorGlueStart]) + penaltyWidth,
        lineStretch: prefixSums.glueStretch[endBreakpoint] - prefixSums.glueStretch[interiorGlueStart],
        lineShrink: prefixSums.glueShrink[endBreakpoint] - prefixSums.glueShrink[interiorGlueStart],
        startOffset: prefixSums.boxWidth[start] + prefixSums.glueWidth[start] - prefixSums.glueShrink[start]
      };
    };
    let minAdjustmentRatioAboveThreshold = Infinity;
    for (let b = 0; b < items.length; b++) {
      const item = items[b];
      if (!isFeasibleBreakpoint(b)) {
        continue;
      }
      let lastActive = null;
      const bestByState = /* @__PURE__ */ new Map();
      active.forEach((a) => {
        let adjustmentRatio = 0;
        const idealLen = lineLen(a.line);
        const { actualLen, lineStretch, lineShrink, startOffset } = lineMetrics(a.index, b);
        if (actualLen < idealLen) {
          adjustmentRatio = (idealLen - actualLen) / lineStretch;
        } else if (actualLen > idealLen) {
          adjustmentRatio = (idealLen - actualLen) / lineShrink;
        } else {
          adjustmentRatio = 0;
        }
        if (adjustmentRatio > currentMaxAdjustmentRatio) {
          minAdjustmentRatioAboveThreshold = Math.min(
            adjustmentRatio,
            minAdjustmentRatioAboveThreshold
          );
        }
        const overfullThreshold = idealLen + startOffset;
        const futureMinFloor = suffixMinBreakpointFloor[b];
        const canUsePruning = (!a.isFallback || !hasNegativeWidths) && !hasNegativeStretchOrShrink;
        if (isForcedBreak(item) || canUsePruning && adjustmentRatio < MIN_ADJUSTMENT_RATIO && futureMinFloor > overfullThreshold) {
          active.delete(a);
          lastActive = a;
        }
        if (adjustmentRatio >= MIN_ADJUSTMENT_RATIO && adjustmentRatio <= currentMaxAdjustmentRatio) {
          let demerits;
          const badness = 100 * Math.abs(adjustmentRatio) ** 3;
          const penalty = item.type === "penalty" ? item.cost : 0;
          if (penalty >= 0) {
            demerits = (1 + badness + penalty) ** 2;
          } else if (penalty > MIN_COST) {
            demerits = (1 + badness) ** 2 - penalty ** 2;
          } else {
            demerits = (1 + badness) ** 2;
          }
          let doubleHyphenPenalty = 0;
          const prevItem = a.index === PARAGRAPH_START ? null : items[a.index];
          if (item.type === "penalty" && prevItem?.type === "penalty") {
            if (item.flagged && prevItem.flagged) {
              doubleHyphenPenalty = opts_.doubleHyphenPenalty;
            }
          }
          demerits += doubleHyphenPenalty;
          let fitness;
          if (adjustmentRatio < -0.5) {
            fitness = 0;
          } else if (adjustmentRatio < 0.5) {
            fitness = 1;
          } else if (adjustmentRatio < 1) {
            fitness = 2;
          } else {
            fitness = 3;
          }
          if (a.index !== PARAGRAPH_START && Math.abs(fitness - a.fitness) > 1) {
            demerits += opts_.adjacentLooseTightPenalty;
          }
          const node = {
            index: b,
            line: a.line + 1,
            fitness,
            isFallback: false,
            totalDemerits: a.totalDemerits + demerits,
            prev: a
          };
          const stateKey = node.line * 4 + fitness;
          const bestNode2 = bestByState.get(stateKey);
          if (!bestNode2 || node.totalDemerits < bestNode2.totalDemerits) {
            bestByState.set(stateKey, node);
          }
        }
      });
      bestByState.forEach((node) => active.add(node));
      if (active.size === 0) {
        if (isFinite(minAdjustmentRatioAboveThreshold)) {
          if (opts_.maxAdjustmentRatio === currentMaxAdjustmentRatio) {
            throw new MaxAdjustmentExceededError();
          }
          return breakLines(items, lineLengths, {
            ...opts,
            initialMaxAdjustmentRatio: minAdjustmentRatioAboveThreshold * 2
          });
        } else {
          active.add({
            index: b,
            line: lastActive.line + 1,
            fitness: 1,
            isFallback: true,
            totalDemerits: lastActive.totalDemerits + 1e3,
            prev: lastActive
          });
        }
      }
    }
    let bestNode = null;
    active.forEach((a) => {
      if (!bestNode || a.totalDemerits < bestNode.totalDemerits) {
        bestNode = a;
      }
    });
    const output = [];
    let next = bestNode;
    while (next) {
      output.push(next.index === PARAGRAPH_START ? 0 : next.index);
      next = next.prev;
    }
    output.reverse();
    return output;
  }
  function adjustmentRatios(items, lineLengths, breakpoints) {
    const lineLen = (i) => Array.isArray(lineLengths) ? lineLengths[i] : lineLengths;
    const ratios = [];
    for (let b = 0; b < breakpoints.length - 1; b++) {
      let idealWidth = lineLen(b);
      let actualWidth = 0;
      let lineShrink = 0;
      let lineStretch = 0;
      const start = lineContentStart(
        items,
        b === 0 ? PARAGRAPH_START : breakpoints[b],
        breakpoints[b + 1]
      );
      for (let p = start; p <= breakpoints[b + 1]; p++) {
        const item = items[p];
        if (item.type === "box") {
          actualWidth += item.width;
        } else if (item.type === "glue" && p !== start && p !== breakpoints[b + 1]) {
          actualWidth += item.width;
          lineShrink += item.shrink;
          lineStretch += item.stretch;
        } else if (item.type === "penalty" && p === breakpoints[b + 1]) {
          actualWidth += item.width;
        }
      }
      let adjustmentRatio;
      if (actualWidth < idealWidth) {
        adjustmentRatio = (idealWidth - actualWidth) / lineStretch;
      } else if (actualWidth > idealWidth) {
        adjustmentRatio = (idealWidth - actualWidth) / lineShrink;
      } else {
        adjustmentRatio = 0;
      }
      ratios.push(adjustmentRatio);
    }
    return ratios;
  }
  function positionItems(items, lineLengths, breakpoints, options = {}) {
    const adjRatios = adjustmentRatios(items, lineLengths, breakpoints);
    const result = [];
    for (let b = 0; b < breakpoints.length - 1; b++) {
      const adjustmentRatio = Math.max(adjRatios[b], MIN_ADJUSTMENT_RATIO);
      let xOffset = 0;
      const start = lineContentStart(
        items,
        b === 0 ? PARAGRAPH_START : breakpoints[b],
        breakpoints[b + 1]
      );
      for (let p = start; p <= breakpoints[b + 1]; p++) {
        const item = items[p];
        if (item.type === "box") {
          result.push({
            item: p,
            line: b,
            xOffset,
            width: item.width
          });
          xOffset += item.width;
        } else if (item.type === "glue" && p !== start && p !== breakpoints[b + 1]) {
          let gap;
          if (adjustmentRatio < 0) {
            gap = item.width + adjustmentRatio * item.shrink;
          } else {
            gap = item.width + adjustmentRatio * item.stretch;
          }
          if (options.includeGlue) {
            result.push({
              item: p,
              line: b,
              xOffset,
              width: gap
            });
          }
          xOffset += gap;
        } else if (item.type === "penalty" && p === breakpoints[b + 1] && item.width > 0) {
          result.push({
            item: p,
            line: b,
            xOffset,
            width: item.width
          });
        }
      }
    }
    return result;
  }
  function forcedBreak() {
    return { type: "penalty", cost: MIN_COST, width: 0, flagged: false };
  }

  return {
    MIN_COST: MIN_COST,
    MAX_COST: MAX_COST,
    PARAGRAPH_START: PARAGRAPH_START,
    MaxAdjustmentExceededError: MaxAdjustmentExceededError,
    breakLines: breakLines,
    forcedBreak: forcedBreak,
    adjustmentRatios: adjustmentRatios,
    positionItems: positionItems,
    lineContentStart: lineContentStart
  };
}
