use hash_collections;

use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, ErrorKind, Write};
use std::collections::LinkedList;

enum Sentence {
    Complete(String),
    Incomplete(String)
}

impl Sentence {
    fn take(self) -> String {
        match self {
            Self::Complete(s) => s,
            Self::Incomplete(s) => s
        }
    }
}

pub struct SentenceIterator {
    reader: BufReader<File>,
    sentences_buffer: LinkedList<Sentence>,
    done: bool
}

impl SentenceIterator {
    pub fn new(file_path: &str) -> Result<Self, io::Error> {
        let file = File::open(file_path)?;
        let mut s = SentenceIterator {
            reader: BufReader::new(file),
            sentences_buffer: LinkedList::new(),
            done: false
        };
        s.sentences_buffer.push_back(Sentence::Incomplete("".to_string()));
        Ok(s)
    }

    pub fn read_sentences(&mut self) {
        if self.done {
            return
        }
        
        let mut line_buffer = 
            if let Some(Sentence::Incomplete(_)) = self.sentences_buffer.back(){
                let mut s = self.sentences_buffer.pop_back().unwrap().take();
                s.push(' ');
                s
            } else {
                String::new()
            };

        if self.reader.read_line(&mut line_buffer).is_ok_and(|b| b > 0) {
            let sentences: Vec<&str> = line_buffer.split(&['.', '?', '!', ';', ':'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
           
            if !sentences.is_empty() { 
                for i in 0..sentences.len()-1 {
                    self.sentences_buffer.push_back({
                        Sentence::Complete(sentences[i].to_string())
                    });
                }

                if line_buffer.trim().ends_with(&['.', '?', '!', ';', ':'][..]){
                    self.sentences_buffer.push_back(Sentence::Complete(sentences[sentences.len()-1].to_string()));
                    self.sentences_buffer.push_back(Sentence::Incomplete("".to_string()));   
                } else {
                    self.sentences_buffer.push_back(Sentence::Incomplete(sentences[sentences.len()-1].to_string()));    
                }
            } else {
                 self.sentences_buffer.push_back(Sentence::Incomplete(line_buffer));   
            }
        } else {
            self.sentences_buffer.push_back(Sentence::Complete(line_buffer));
            self.done = true;
        }
    }
}

impl Iterator for SentenceIterator {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(Sentence::Incomplete(_)) = self.sentences_buffer.front() {
                self.read_sentences();
            } else {
                break;
            }
        }
        self.sentences_buffer.pop_front().map(|s| s.take())
    }
}

fn prompt_for_input(prompt: &str) -> String {
    print!("{} > ", prompt);
    io::stdout().flush().unwrap();
    let mut input_string = String::new();
    io::stdin().read_line(&mut input_string).unwrap();
    input_string.trim().to_string()
}


fn main() -> Result<(), Box<dyn Error>> {
    let input_file_path = prompt_for_input("Enter file name");
    if input_file_path.len() == 0 {
        println!("No file name provided. Terminating!!");
        return Err(std::io::Error::new(ErrorKind::InvalidFilename, "file name not provided").into())
    }

    let mut graph = hash_collections::FixedSizeHashGraphMap::<String, bool, 50849>::new();

    println!("Parsing file ...");
    let sentence_iter = SentenceIterator::new(input_file_path.as_str())?;
    for sentence in sentence_iter {
        graph.insert((sentence.clone(), true), vec![])?;
        
        let words: Vec<&str> = sentence.split(&[' ', ',' , '"'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        for i in 0..words.len() {
            if i + 1 < words.len() {
                //println!("adding {} / {}", words[i], words[i+1]);
                graph.insert(
                    (words[i].to_string(), false),
                    vec![(words[i+1].to_string(), false)]
                )?
            } else {
                graph.insert(
                    (words[i].to_string(), false),
                    vec![]
                )?
            }

            graph.connect_to(&words[i].to_string(), vec![&sentence]);
        }
    }

    loop {
        let input_word = prompt_for_input("Enter a word");
        if input_word.len() > 0 {
            if let Some(node) = graph.node(&input_word) {
                println!("  '{}' connected to words:", input_word);
                for (word_node, w) in node.iter_out_edges().filter(|(n, _)| *n.value()==false){
                    println!("    {}, ({})", word_node.key(), w);
                }

                println!("  '{}' found in sentences:", input_word);
                for (word_node, w) in node.iter_out_edges().filter(|(n, _)| *n.value()==true){
                    println!("    {}, ({})", word_node.key(), w);
                }
            } else {
                println!("Word '{}' not in book. Try another one.", input_word);
            }
        } else {
            break;
        }
    }

    println!("Ok bye!!");
    Ok(())
}

