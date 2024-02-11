document.addEventListener("DOMContentLoaded", function() {
  const beans = document.querySelectorAll('.bean');

  beans.forEach(item => {
    item.addEventListener('click', function() {
      const id = this.getAttribute('data-toggle');
      const elem = document.getElementById(id);
      const toshow = elem.classList.contains('hidden');
      document.querySelectorAll('.details').forEach(c => {
        c.classList.add('hidden');
      });
      if (toshow) {
        elem.classList.remove('hidden');
      }
    });
  });
});
