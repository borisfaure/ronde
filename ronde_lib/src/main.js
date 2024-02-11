document.addEventListener("DOMContentLoaded", function() {
  const beans = document.querySelectorAll('.bean');

  beans.forEach(item => {
    item.addEventListener('click', function() {
      const id = this.getAttribute('data-toggle');
      const elem = document.getElementById(contentId);
      if elem.classList.contains('hidden') {
        elem.classList.remove('hidden');
      } else {
        elem.classList.add('hidden');
      }
    });
  });
});
