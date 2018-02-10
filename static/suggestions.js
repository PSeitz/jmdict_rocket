function ready(fn) {
  if (document.attachEvent ? document.readyState === "complete" : document.readyState !== "loading"){
    fn();
  } else {
    document.addEventListener('DOMContentLoaded', fn);
  }
}

var suggest_request = new XMLHttpRequest();

function get_json(url, cb) {

    suggest_request.open('GET', url, true);

    suggest_request.onload = function() {
        if (this.status >= 200 && this.status < 400) {
            // Success!
            var data = JSON.parse(this.response);
            cb(data)
        } else {
            // We reached our target server, but it returned an error
        }
    };

    suggest_request.onerror = function() {
        // There was a connection error of some sort
    };

    suggest_request.send();

}

function get_input() {
    return document.getElementById("input_field")
}


function get_suggest_html(lis) {

    lis = lis.map(function(el){
        return '<li class="list_entry"> '+el[0]+' </li>'
    }).join('\n')

    return `<div class="el-autocomplete-suggestion" style="width: 100%;"  id="suggestion_div">
       <div class="el-scrollbar">
          <div class="el-autocomplete-suggestion__wrap el-scrollbar__wrap" style="margin-bottom: -15px; margin-right: -15px;">
             <ul class="el-scrollbar__view el-autocomplete-suggestion__list" style="position: relative;">
                ${lis}
             </ul>
          </div>
          <div class="el-scrollbar__bar is-horizontal">
             <div class="el-scrollbar__thumb" style="transform: translateX(0%);"></div>
          </div>
          <div class="el-scrollbar__bar is-vertical">
             <div class="el-scrollbar__thumb" style="transform: translateY(0%);"></div>
          </div>
       </div>
    </div>`

}


// highlight_suggest(index) {
//     if (!this.suggestionVisible || this.loading) { return; }
//     if (index < 0) index = 0;
//     if (index >= this.suggestions.length) {
//       index = this.suggestions.length - 1;
//     }
//     const suggestion = this.$refs.suggestions.$el.querySelector('.el-autocomplete-suggestion__wrap');
//     const suggestionList = suggestion.querySelectorAll('.el-autocomplete-suggestion__list li');

//     let highlightItem = suggestionList[index];
//     let scrollTop = suggestion.scrollTop;
//     let offsetTop = highlightItem.offsetTop;

//     if (offsetTop + highlightItem.scrollHeight > (scrollTop + suggestion.clientHeight)) {
//       suggestion.scrollTop += highlightItem.scrollHeight;
//     }
//     if (offsetTop < scrollTop) {
//       suggestion.scrollTop -= highlightItem.scrollHeight;
//     }

//     this.highlightedIndex = index;
//     // this.inputValue = this.suggestions[this.highlightedIndex].value
//     // this.$emit('input', this.value);
//     // this.$emit('input', this.suggestions[this.highlightedIndex].value);
// }


ready(function(){

    var x = get_input();
    x.addEventListener("focus", myFocusFunction, true);
    x.addEventListener("blur", myBlurFunction, false);
    x.addEventListener("keydown", keypressed, true);
    x.oninput = input;

    function myFocusFunction() {
        document.getElementById("suggestion_div").style.display = '';
    }

    function myBlurFunction() {
        // setTimeout(function(){
        //     document.getElementById("suggestion_div").style.display = 'none';
        // },1)
        
    }

    function keypressed(e) {
        if (e.keyCode == 13) {
            console.log("enter" + get_input().value)
            window.location = "http://localhost:8000/?skip=0&q="+get_input().value.trim();
            return false;
        }
    }
    function input(e) {
        get_json('/suggest?q='+get_input().value, function(data){
            console.time("append suggestions");
            var el = document.getElementById("suggestion_div")
            el.outerHTML = get_suggest_html(data);
            el.style.display = '';

            var elements = document.querySelectorAll('.list_entry');
            Array.prototype.forEach.call(elements, function(el, i){
                el.addEventListener("click", function( event ) {
                    window.location = "http://localhost:8000/?skip=0&q="+el.textContent.trim();
                }, false);
            });


            console.timeEnd("append suggestions");
            // console.log(data)
        })
    }

})






