import logging
import azure.functions as func
import os
#os.environ['TF_CPP_MIN_LOG_LEVEL'] = '3' 
from allennlp.predictors.predictor import Predictor
import json

logging.info('Starting General Bots Models server.')

# https://ai.google.com/research/NaturalQuestions

predictor = None

from flask import Flask, render_template, request
import hmac



app = Flask(__name__)


@app.route("/reading-comprehension",  methods=['POST'])
def index():    
    logging.info('General Bots QA.')

    content = request.form.get('content')
    question = request.args.get('question')
    key = request.args.get('key')

    if not hmac.compare_digest(key, 'starter'):
        return 'Invalid key.'

    global predictor
    if predictor is None:
        predictor = Predictor.from_path("https://storage.googleapis.com/allennlp-public-models/transformer-qa-2020-10-03.tar.gz")
    
    answer = predictor.predict(
        passage=content,
        question=question
    )['best_span_str']
       

    if answer:
        return answer
    else:
        return "No answers for this question."
    
app.run(debug=True)

        