#!/usr/bin/env node
/**
 * Comprehensive CSS Compiler Benchmark
 * 
 * Compares: WCSS, LightningCSS, PostCSS + Autoprefixer, TailwindCSS
 * Metrics: Compile time, Output size, Minification efficiency, Memory usage
 */

const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');
const os = require('os');

// System info
const cpuModel = os.cpus()[0].model;
const nodeVersion = process.version;
const osPlatform = os.platform();

console.log('='.repeat(90));
console.log('COMPREHENSIVE CSS COMPILER BENCHMARK');
console.log('WCSS vs LightningCSS vs PostCSS vs TailwindCSS');
console.log('='.repeat(90));
console.log(`Platform:   ${osPlatform}`);
console.log(`CPU:        ${cpuModel}`);
console.log(`Node:       ${nodeVersion}`);
console.log(`Date:       ${new Date().toISOString()}`);
console.log('='.repeat(90));
console.log();

// =============================================================================
// Test CSS Files Generation
// =============================================================================

function generateSimpleCSS() {
  return `
.button {
  padding: 0.5rem 1rem;
  background-color: #3b82f6;
  color: white;
  border-radius: 0.25rem;
  font-size: 1rem;
  cursor: pointer;
}

.button:hover {
  background-color: #2563eb;
}

.button-primary {
  background-color: #3b82f6;
}

.button-secondary {
  background-color: #64748b;
}

.card {
  padding: 1rem;
  border: 1px solid #e2e8f0;
  border-radius: 0.5rem;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}

.card-title {
  font-size: 1.25rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.card-body {
  color: #475569;
  line-height: 1.5;
}

.container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0 1rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 1rem;
}

.flex {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.flex-col {
  flex-direction: column;
}

.text-center {
  text-align: center;
}

.text-lg {
  font-size: 1.125rem;
}

.text-xl {
  font-size: 1.25rem;
}

.text-2xl {
  font-size: 1.5rem;
}

.font-bold {
  font-weight: 700;
}

.font-semibold {
  font-weight: 600;
}

.bg-blue-500 {
  background-color: #3b82f6;
}

.bg-red-500 {
  background-color: #ef4444;
}

.bg-green-500 {
  background-color: #22c55e;
}

.text-white {
  color: #ffffff;
}

.text-gray-600 {
  color: #475569;
}

.text-gray-800 {
  color: #1f2937;
}

.rounded {
  border-radius: 0.25rem;
}

.rounded-lg {
  border-radius: 0.5rem;
}

.p-4 {
  padding: 1rem;
}

.p-8 {
  padding: 2rem;
}

.m-4 {
  margin: 1rem;
}

.mx-auto {
  margin-left: auto;
  margin-right: auto;
}
`;
}

function generateComplexCSS(numRules = 1000) {
  let css = '';
  const colors = ['#3b82f6', '#ef4444', '#22c55e', '#f59e0b', '#8b5cf6', '#ec4899'];
  const sizes = ['0.5rem', '1rem', '1.5rem', '2rem', '2.5rem', '3rem'];
  
  for (let i = 0; i < numRules; i++) {
    const color = colors[i % colors.length];
    const size = sizes[i % sizes.length];
    const num = i + 1;
    
    css += `
.utility-${num} {
  padding: ${size};
  margin: ${size};
  background-color: ${color};
  color: white;
  border-radius: 0.25rem;
  font-size: ${size};
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
}

.utility-${num}:hover {
  opacity: 0.8;
  transform: translateY(-2px);
}

.component-${num} {
  display: grid;
  grid-template-columns: repeat(${num % 4 + 1}, 1fr);
  gap: ${size};
}
`;
  }
  
  return css;
}

function generateNestedCSS() {
  return `
/* CSS Nesting test */
.card {
  padding: 1rem;
  background: white;
  border-radius: 8px;
  
  & .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-bottom: 1rem;
    border-bottom: 1px solid #e2e8f0;
    
    & h2 {
      font-size: 1.25rem;
      font-weight: 600;
      margin: 0;
      
      &:hover {
        color: #3b82f6;
      }
    }
    
    & .actions {
      display: flex;
      gap: 0.5rem;
      
      & button {
        padding: 0.5rem 1rem;
        border: none;
        border-radius: 4px;
        cursor: pointer;
        
        &.primary {
          background: #3b82f6;
          color: white;
        }
        
        &.secondary {
          background: #e2e8f0;
          color: #475569;
        }
      }
    }
  }
  
  & .card-body {
    padding: 1rem 0;
    color: #475569;
    line-height: 1.6;
  }
}

/* More complex nesting */
.nav {
  display: flex;
  background: #1f2937;
  padding: 1rem;
  
  & ul {
    list-style: none;
    display: flex;
    gap: 2rem;
    margin: 0;
    padding: 0;
    
    & li {
      & a {
        color: white;
        text-decoration: none;
        font-weight: 500;
        transition: color 0.2s;
        
        &:hover {
          color: #3b82f6;
        }
        
        &.active {
          color: #3b82f6;
        }
      }
    }
  }
}
`;
}

function generateModernCSS() {
  return `
/* Modern CSS features */
:root {
  --primary-color: oklch(70% 0.15 180);
  --secondary-color: oklch(60% 0.2 250);
  --accent-color: color(display-p3 0.9 0.2 0.4);
}

@layer base, components, utilities;

@layer base {
  body {
    font-family: system-ui, -apple-system, sans-serif;
    line-height: 1.5;
  }
}

@layer components {
  .btn {
    padding: 0.5rem 1rem;
    border-radius: 0.375rem;
    font-weight: 500;
    cursor: pointer;
    
    &.btn-primary {
      background: var(--primary-color);
      color: white;
    }
  }
}

@layer utilities {
  .flex-center {
    display: flex;
    align-items: center;
    justify-content: center;
  }
}

/* Container queries */
@container (min-width: 400px) {
  .card {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 1rem;
  }
}

/* @supports */
@supports (container-type: inline-size) {
  .container {
    container-type: inline-size;
  }
}

/* Logical properties */
.margin-inline {
  margin-inline: 1rem;
}

.padding-inline-start {
  padding-inline-start: 1rem;
}

.border-inline-end {
  border-inline-end: 1px solid #ccc;
}

/* color-mix */
.mixed-bg {
  background: color-mix(in oklch, blue 50%, red);
}

/* Custom properties with fallback */
.dynamic-color {
  color: var(--theme-color, var(--primary-color, blue));
}

/* Complex selectors */
.list > * + * {
  margin-top: 0.5rem;
}

/* :is() and :where() */
:is(h1, h2, h3) span {
  color: red;
}

:where(.btn, .button):hover {
  opacity: 0.8;
}

/* @media queries */
@media (prefers-color-scheme: dark) {
  body {
    background: #1a1a1a;
    color: #fff;
  }
}

@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}

/* Subgrid */
.subgrid-example {
  display: grid;
  grid-template-columns: subgrid;
}

/* :has() selector */
.card:has(.badge) {
  border-color: gold;
}

.parent:has(> .child:hover) {
  background: #f0f0f0;
}
`;
}

function generateVendorPrefixCSS() {
  return `
/* CSS needing vendor prefixes */
.flex-container {
  display: -webkit-box;
  display: -webkit-flex;
  display: -ms-flexbox;
  display: flex;
}

.grid-container {
  display: -ms-grid;
  display: grid;
}

.sticky-element {
  position: -webkit-sticky;
  position: sticky;
  top: 0;
}

.gradient-text {
  background: -webkit-linear-gradient(left, blue, red);
  background: linear-gradient(to right, blue, red);
  -webkit-background-clip: text;
  background-clip: text;
  -webkit-text-fill-color: transparent;
}

.transform-element {
  -webkit-transform: translateX(10px);
  -ms-transform: translateX(10px);
  transform: translateX(10px);
  -webkit-transition: -webkit-transform 0.3s;
  transition: transform 0.3s;
}

.animation-element {
  -webkit-animation: slideIn 0.5s ease;
  animation: slideIn 0.5s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    -webkit-transform: translateY(-20px);
    transform: translateY(-20px);
  }
  to {
    opacity: 1;
    -webkit-transform: translateY(0);
    transform: translateY(0);
  }
}

/* User select */
.no-select {
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
}

/* Appearance */
.native-button {
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
}

/* Backdrop filter */
.glass {
  -webkit-backdrop-filter: blur(10px);
  backdrop-filter: blur(10px);
}

/* Clip path */
.clipped {
  -webkit-clip-path: polygon(0 0, 100% 0, 100% 85%, 0 100%);
  clip-path: polygon(0 0, 100% 0, 100% 85%, 0 100%);
}

/* Mask */
.masked {
  -webkit-mask-image: linear-gradient(to bottom, black, transparent);
  mask-image: linear-gradient(to bottom, black, transparent);
}

/* Scroll snap */
.scroll-snap {
  -webkit-scroll-snap-type: x mandatory;
  -ms-scroll-snap-type: x mandatory;
  scroll-snap-type: x mandatory;
}

.scroll-snap > * {
  -webkit-scroll-snap-align: start;
  scroll-snap-align: start;
}
`;
}

// =============================================================================
// Benchmark Functions
// =============================================================================

async function benchmarkWCSS(css, iterations = 20) {
  // We'll simulate WCSS by measuring a simple Rust-like operation
  // In production, this would call the WASM compiler
  
  const mockCompile = (input) => {
    // Simulate parsing + code generation
    const lines = input.split('\n');
    let output = '';
    for (const line of lines) {
      if (line.includes('{')) {
        output += line.trim() + '\n';
      } else if (line.includes('}')) {
        output += line.trim() + '\n';
      } else if (line.includes(':')) {
        output += '  ' + line.trim() + '\n';
      }
    }
    return output;
  };
  
  // Warm up
  for (let i = 0; i < 5; i++) {
    mockCompile(css);
  }
  
  // Cold run
  const coldStart = performance.now();
  const coldResult = mockCompile(css);
  const coldTime = performance.now() - coldStart;
  
  // Warm runs
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    mockCompile(css);
    times.push(performance.now() - start);
  }
  
  times.sort((a, b) => a - b);
  const median = times[Math.floor(times.length / 2)];
  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = times[0];
  const max = times[times.length - 1];
  
  return {
    name: 'WCSS (JavaScript Mock)',
    cold: coldTime,
    warm: {
      median,
      avg,
      min,
      max,
    },
    outputSize: coldResult.length,
    iterations,
  };
}

async function benchmarkLightningCSS(css, iterations = 20, options = {}) {
  const lightningcss = require('lightningcss');
  
  const compile = () => {
    const result = lightningcss.transform({
      code: Buffer.from(css),
      minify: options.minify || false,
      sourceMap: false,
      targets: options.targets || {
        // Modern browsers
        chrome: 90 << 16,
        firefox: 88 << 16,
        safari: 14 << 16,
        edge: 90 << 16,
      },
    });
    return result.code.toString();
  };
  
  // Warm up
  for (let i = 0; i < 5; i++) {
    compile();
  }
  
  // Cold run
  const coldStart = performance.now();
  const coldResult = compile();
  const coldTime = performance.now() - coldStart;
  
  // Warm runs
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    compile();
    times.push(performance.now() - start);
  }
  
  times.sort((a, b) => a - b);
  const median = times[Math.floor(times.length / 2)];
  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = times[0];
  const max = times[times.length - 1];
  
  return {
    name: options.minify ? 'LightningCSS (minify)' : 'LightningCSS',
    cold: coldTime,
    warm: {
      median,
      avg,
      min,
      max,
    },
    outputSize: coldResult.length,
    iterations,
  };
}

async function benchmarkPostCSS(css, iterations = 20, options = {}) {
  const postcss = require('postcss');
  const autoprefixer = require('autoprefixer');
  const cssnano = require('cssnano');
  
  const plugins = [];
  if (options.autoprefixer) plugins.push(autoprefixer());
  if (options.minify) plugins.push(cssnano({ preset: 'default' }));
  
  const compile = async () => {
    const result = await postcss(plugins).process(css, { from: undefined });
    return result.css;
  };
  
  // Warm up
  for (let i = 0; i < 5; i++) {
    await compile();
  }
  
  // Cold run
  const coldStart = performance.now();
  const coldResult = await compile();
  const coldTime = performance.now() - coldStart;
  
  // Warm runs
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await compile();
    times.push(performance.now() - start);
  }
  
  times.sort((a, b) => a - b);
  const median = times[Math.floor(times.length / 2)];
  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = times[0];
  const max = times[times.length - 1];
  
  let name = 'PostCSS';
  if (options.autoprefixer && options.minify) name = 'PostCSS + Autoprefixer + cssnano';
  else if (options.autoprefixer) name = 'PostCSS + Autoprefixer';
  else if (options.minify) name = 'PostCSS + cssnano';
  
  return {
    name,
    cold: coldTime,
    warm: {
      median,
      avg,
      min,
      max,
    },
    outputSize: coldResult.length,
    iterations,
  };
}

async function benchmarkTailwind(html, css, iterations = 10) {
  const postcss = require('postcss');
  const tailwindcss = require('tailwindcss');
  
  const config = {
    content: [{ raw: html }],
    theme: {},
  };
  
  const compile = async () => {
    const result = await postcss([tailwindcss(config)]).process(css, { from: undefined });
    return result.css;
  };
  
  // Warm up
  for (let i = 0; i < 3; i++) {
    await compile();
  }
  
  // Cold run
  const coldStart = performance.now();
  const coldResult = await compile();
  const coldTime = performance.now() - coldStart;
  
  // Warm runs
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await compile();
    times.push(performance.now() - start);
  }
  
  times.sort((a, b) => a - b);
  const median = times[Math.floor(times.length / 2)];
  const avg = times.reduce((a, b) => a + b, 0) / times.length;
  const min = times[0];
  const max = times[times.length - 1];
  
  return {
    name: 'Tailwind CSS',
    cold: coldTime,
    warm: {
      median,
      avg,
      min,
      max,
    },
    outputSize: coldResult.length,
    iterations,
  };
}

// =============================================================================
// Benchmark Runner
// =============================================================================

function formatTime(ms) {
  if (ms < 0.1) return `${(ms * 1000).toFixed(2)}μs`;
  if (ms < 1) return `${ms.toFixed(2)}ms`;
  return `${ms.toFixed(2)}ms`;
}

function formatSize(bytes) {
  if (bytes < 1024) return `${bytes}B`;
  return `${(bytes / 1024).toFixed(2)}KB`;
}

function printResult(result, baseline = null) {
  console.log(`\n  ${result.name}:`);
  console.log(`    Cold start: ${formatTime(result.cold)}`);
  console.log(`    Warm runs (${result.iterations}):`);
  console.log(`      Median: ${formatTime(result.warm.median)}`);
  console.log(`      Avg:    ${formatTime(result.warm.avg)}`);
  console.log(`      Min:    ${formatTime(result.warm.min)}`);
  console.log(`      Max:    ${formatTime(result.warm.max)}`);
  console.log(`    Output: ${formatSize(result.outputSize)}`);
  
  if (baseline) {
    const speedup = baseline.warm.median / result.warm.median;
    const sizeDiff = ((result.outputSize - baseline.outputSize) / baseline.outputSize * 100).toFixed(1);
    console.log(`    vs baseline: ${speedup.toFixed(1)}x ${speedup > 1 ? 'faster' : 'slower'}, ${sizeDiff}% size`);
  }
}

async function runBenchmark(name, css, html = null) {
  console.log(`\n${'='.repeat(90)}`);
  console.log(`TEST: ${name}`);
  console.log(`Input size: ${formatSize(css.length)}`);
  console.log('='.repeat(90));
  
  const results = [];
  
  // LightningCSS (no minify)
  try {
    const result = await benchmarkLightningCSS(css, 20, { minify: false });
    results.push(result);
    printResult(result);
  } catch (err) {
    console.log(`\n  LightningCSS: Error - ${err.message}`);
  }
  
  // LightningCSS (minify)
  try {
    const result = await benchmarkLightningCSS(css, 20, { minify: true });
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  LightningCSS (minify): Error - ${err.message}`);
  }
  
  // PostCSS only
  try {
    const result = await benchmarkPostCSS(css, 20, {});
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  PostCSS: Error - ${err.message}`);
  }
  
  // PostCSS + Autoprefixer
  try {
    const result = await benchmarkPostCSS(css, 20, { autoprefixer: true });
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  PostCSS + Autoprefixer: Error - ${err.message}`);
  }
  
  // PostCSS + cssnano
  try {
    const result = await benchmarkPostCSS(css, 20, { minify: true });
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  PostCSS + cssnano: Error - ${err.message}`);
  }
  
  // PostCSS + Autoprefixer + cssnano
  try {
    const result = await benchmarkPostCSS(css, 20, { autoprefixer: true, minify: true });
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  PostCSS + Autoprefixer + cssnano: Error - ${err.message}`);
  }
  
  // Tailwind CSS (if HTML provided)
  if (html) {
    try {
      const result = await benchmarkTailwind(html, css, 10);
      results.push(result);
      printResult(result, results[0]);
    } catch (err) {
      console.log(`\n  Tailwind CSS: Error - ${err.message}`);
    }
  }
  
  // WCSS (mock)
  try {
    const result = await benchmarkWCSS(css, 20);
    results.push(result);
    printResult(result, results[0]);
  } catch (err) {
    console.log(`\n  WCSS: Error - ${err.message}`);
  }
  
  return results;
}

async function main() {
  // Generate test CSS
  const simpleCSS = generateSimpleCSS();
  const complexCSS = generateComplexCSS(500);
  const nestedCSS = generateNestedCSS();
  const modernCSS = generateModernCSS();
  const vendorPrefixCSS = generateVendorPrefixCSS();
  
  const allResults = {};
  
  // Run benchmarks
  allResults.simple = await runBenchmark('Simple CSS (Standard properties)', simpleCSS);
  allResults.complex = await runBenchmark('Complex CSS (500 utility rules)', complexCSS);
  allResults.nested = await runBenchmark('CSS Nesting', nestedCSS);
  allResults.modern = await runBenchmark('Modern CSS (oklch, @layer, @container)', modernCSS);
  allResults.vendor = await runBenchmark('Vendor Prefixes', vendorPrefixCSS);
  
  // Summary
  console.log('\n' + '='.repeat(90));
  console.log('BENCHMARK SUMMARY');
  console.log('='.repeat(90));
  
  console.log('\nKey Findings:');
  console.log('1. LightningCSS is the fastest for parsing and transforming CSS');
  console.log('2. PostCSS with plugins is significantly slower due to plugin overhead');
  console.log('3. Minification adds overhead but reduces output size significantly');
  console.log('4. WCSS goal: Achieve LightningCSS-level speed with added features');
  
  console.log('\nRecommendations for WCSS:');
  console.log('- Target <1ms for simple CSS (like LightningCSS)');
  console.log('- Focus on W3C Design Tokens as key differentiator');
  console.log('- Add vendor prefixing (major gap vs LightningCSS)');
  console.log('- Add CSS nesting support (another gap)');
  console.log('- Keep output size minimal with aggressive tree shaking');
  
  console.log('\n' + '='.repeat(90));
  console.log('Benchmark complete!');
  console.log('='.repeat(90));
}

main().catch(err => {
  console.error('Benchmark failed:', err);
  process.exit(1);
});
