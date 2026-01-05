# On the use of LLMs in this Project

This document explains why and how LLMs were used in the development of eilmeldung, and shares lessons learned from this experiment in learning Rust through LLM assistance.

---

## Table of Contents

- [Background](#background)
- [Some Context](#some-context)
- [How LLMs were used in this Project](#how-llms-were-used-in-this-project)
- [How LLMs were NOT used in this Project](#how-llms-were-not-used-in-this-project)
- [Purely Anecdotal Lessons Learned](#purely-anecdotal-lessons-learned)
- [Key Takeaway](#key-takeaway)
- [Tools and Prompts](#tools-and-prompts)

---

## Background

This project was built as an experiment in learning Rust through LLM use.

---

## Some Context

I teach programming/computer science at a university of applied sciences. Over the last few years, I've witnessed a change in how students *learn* and *understand* programming and related concepts by using LLMs. While for some students, using LLMs brings real benefits, for others it becomes a crutch that prevents genuine learning. The difference lies not in the tool itself, but in *how* it's used. I am not only talking about *cheating* in assignments. The main problems are in my opinion:

1. LLMs are trained to produce code and solve problems: when a student encounters a problem, LLMs tend to produce code, preventing students from overcoming the challenge themselves and robbing them of a vital learning opportunity.
2. As LLMs tend to be sycophantic and pleasing in the nature of their answers, students fall into the trap of believing that they understood the concept under investigation. This may be true on a conceptual level. However, programming is a *doing art* which is only understood when students overcome the challenge of *applying a programming concept* (by failing and then succeeding).

Consider this analogy I sometimes use with my students: You want to learn to swim. An LLM can explain the mechanics—how to move your arms, when to breathe, how to stay afloat. But would you then jump into deep water based solely on that explanation? Of course not. You'd need hours of practice in shallow water, struggling, failing, and gradually improving.

Programming is no different. Yet LLMs make it tempting to skip the struggle entirely. To be fair, the same argument applies to any passive learning method (like YouTube videos or classical lectures). However, never has this approach of purely conceptual learning been so alluring as with LLMs.

That said, LLMs, it seems at the moment, are here to stay. Knowing how to use them (and when not) is a vital ability which already plays a certain role in programming. For this reason I am incorporating "developing using LLMs" into my programming course ("Advanced topics in Java"). In order to make sure to really understand what I am talking about, I needed to apply LLMs to *learn a new programming language* myself. And this project *eilmeldung* is the result of this endeavour.

---

## How LLMs were used in this Project

LLMs were used in this project to understand if and how they can be used for the following:

- **Learning a new programming language or concept using a *Tutor Agent Prompt***: The tutor agent prompt tells the LLM to *not produce any solutions* or *code*. Instead, the LLM was prompted to lead me to a solution by asking questions. This approach was applied also to compiler errors.
- **Explaining existing code bases using a *Explainer Agent Prompt***: With this prompt, the LLM explains existing code bases to more quickly understand *idiomatic programming approaches* and *architectures*.
- **Creating documentation** (e.g., [Commands](commands.md))
- **Refactoring after a certain pattern**: After refactoring one or more modules, the LLM was asked to refactor remaining modules in a similar manner.
- **Creating fine-grained commits**.

---

## How LLMs were NOT used in this Project

This project is **not vibe-coded**. Every line was intentionally written to solve a problem I understood. The code has *warts*, i.e., awkward Rust patterns, over-engineered solutions, remnants of learning mistakes --- and that's the point. This is what *learning* looks like.

---

## Purely Anecdotal Lessons Learned

- **Using an LLM as a tutor was mostly successful**. For example, when debugging borrow checker errors, having the LLM *ask* me questions like "What is the lifetime of this reference?" was far more educational than receiving a corrected code snippet. However, as the context becomes longer, LLMs tend to forget their role as tutors (*context rot*) and start to produce code again. Apart from that, LLMs *can* be really good sparring partners when it comes to learning.
- **Explaining unknown code bases works relatively well** as long as the code base is not too large and questions are either very specific or very high-level.
- **Creating documentation works but needs to be checked *very carefully*** for errors and wrong assumptions.
- **Refactoring (in my case) didn't work** and I had to revert the changes: LLMs tend to produce code which is not very maintainable and does not fit to the existing architecture.
- **Committing worked well at first but led to data loss in one case** (LLM stashed all changes, then dropped the stash and then tried to re-implement the changes itself)

---

## Key Takeaway 

If you're learning with LLMs:
- **Do**: Use them as tutors that ask questions, not answer machines
- **Do**: Implement solutions yourself, even when LLMs offer code
- **Don't**: Let LLMs rob you of the struggle --- the struggle *is* the learning
- **Don't**: Mistake understanding an explanation for having the skill

---

## Tools and Prompts

I am using [neovim](https://neovim.io/) with [opencode](https://opencode.ai/). Here are the prompts in *opencode agent format* I've developed for the different tasks:

- [tutor.md](https://github.com/user-attachments/files/24354660/tutor.md): Tutor helping to understand new concepts
- [explainer.md](https://github.com/user-attachments/files/24354664/explainer.md): For understanding code bases
- [unit-tester.md](https://github.com/user-attachments/files/24354665/unit-tester.md): For creating unit test. Note how the LLM should **deny** creating tests on implementations or unclear specification.
