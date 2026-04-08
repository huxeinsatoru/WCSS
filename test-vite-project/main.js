import styles from './styles.euis';

// Inject the styles into the page
const styleElement = document.createElement('style');
styleElement.textContent = styles;
document.head.appendChild(styleElement);

console.log('Euis styles loaded successfully!');
console.log('CSS length:', styles.length, 'characters');

// Add some interactivity
document.querySelector('.button').addEventListener('click', () => {
  alert('Button clicked! Euis is working correctly.');
});
